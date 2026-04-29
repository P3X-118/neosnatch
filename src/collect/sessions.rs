use anyhow::Result;
use chrono::{DateTime, Local, TimeZone};
use std::ffi::CStr;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;

#[derive(Debug, Clone)]
pub struct Session {
    pub user: String,
    pub line: String,
    pub host: Option<String>,
    pub when: Option<String>,
}

// utmp/wtmp record layout (glibc utmpx, Linux x86_64).
// Sized to 384 bytes; we parse only what we need.
const UT_LINESIZE: usize = 32;
const UT_NAMESIZE: usize = 32;
const UT_HOSTSIZE: usize = 256;

const USER_PROCESS: i16 = 7;

#[repr(C)]
struct ExitStatus { _e_termination: i16, _e_exit: i16 }

#[repr(C)]
struct TimeVal { tv_sec: i32, tv_usec: i32 }

#[repr(C)]
struct Utmpx {
    ut_type: i16,
    _pad: [u8; 2],
    _ut_pid: i32,
    ut_line: [u8; UT_LINESIZE],
    _ut_id: [u8; 4],
    ut_user: [u8; UT_NAMESIZE],
    ut_host: [u8; UT_HOSTSIZE],
    _ut_exit: ExitStatus,
    _ut_session: i32,
    ut_tv: TimeVal,
    _ut_addr_v6: [i32; 4],
    _unused: [u8; 20],
}

const UTMPX_SIZE: usize = size_of::<Utmpx>();

fn read_records(path: &str) -> Result<Vec<Utmpx>> {
    let mut f = File::open(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let mut out = Vec::with_capacity(buf.len() / UTMPX_SIZE);
    let mut i = 0;
    while i + UTMPX_SIZE <= buf.len() {
        let mut rec: Utmpx = unsafe { std::mem::zeroed() };
        let dst = unsafe {
            std::slice::from_raw_parts_mut(&mut rec as *mut _ as *mut u8, UTMPX_SIZE)
        };
        dst.copy_from_slice(&buf[i..i + UTMPX_SIZE]);
        out.push(rec);
        i += UTMPX_SIZE;
    }
    Ok(out)
}

fn cstr(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).trim().to_string()
}

fn fmt_time(secs: i32) -> String {
    Local.timestamp_opt(secs as i64, 0).single()
        .map(|dt: DateTime<Local>| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default()
}

fn record_to_session(r: &Utmpx) -> Session {
    let host = cstr(&r.ut_host);
    Session {
        user: cstr(&r.ut_user),
        line: cstr(&r.ut_line),
        host: if host.is_empty() { None } else { Some(host) },
        when: Some(fmt_time(r.ut_tv.tv_sec)),
    }
}

pub fn active() -> Result<Vec<Session>> {
    let recs = match read_records("/var/run/utmp") {
        Ok(r) => r,
        Err(_) => return Ok(Vec::new()),
    };
    Ok(recs.iter()
        .filter(|r| r.ut_type == USER_PROCESS)
        .filter(|r| !cstr(&r.ut_user).is_empty())
        .map(record_to_session)
        .collect())
}

pub fn last() -> Result<Option<Session>> {
    // wtmp can be many MB; read backward in fixed-size chunks instead of
    // slurping the whole file. Stop at the first USER_PROCESS record.
    let mut f = match File::open("/var/log/wtmp") {
        Ok(f) => f,
        Err(_) => return Ok(None),
    };
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    if len < UTMPX_SIZE as u64 { return Ok(None); }

    let mut offset = (len / UTMPX_SIZE as u64) * UTMPX_SIZE as u64;
    let mut buf = [0u8; UTMPX_SIZE];
    while offset >= UTMPX_SIZE as u64 {
        offset -= UTMPX_SIZE as u64;
        f.seek(SeekFrom::Start(offset))?;
        f.read_exact(&mut buf)?;
        let mut rec: Utmpx = unsafe { std::mem::zeroed() };
        let dst = unsafe {
            std::slice::from_raw_parts_mut(&mut rec as *mut _ as *mut u8, UTMPX_SIZE)
        };
        dst.copy_from_slice(&buf);
        if rec.ut_type == USER_PROCESS && !cstr(&rec.ut_user).is_empty() {
            return Ok(Some(record_to_session(&rec)));
        }
    }
    Ok(None)
}

/// Hosts that have appeared in wtmp at least `min_count` times. Used by the
/// snapshot generator to seed the "known IPs" list for anomaly detection.
pub fn known_hosts(min_count: usize) -> Result<Vec<String>> {
    use std::collections::HashMap;
    let recs = match read_records("/var/log/wtmp") {
        Ok(r) => r,
        Err(_) => return Ok(Vec::new()),
    };
    let mut counts: HashMap<String, usize> = HashMap::new();
    for r in recs.iter() {
        if r.ut_type != USER_PROCESS { continue; }
        let host = cstr(&r.ut_host);
        if host.is_empty() { continue; }
        *counts.entry(host).or_insert(0) += 1;
    }
    let mut out: Vec<String> = counts.into_iter()
        .filter(|(_, n)| *n >= min_count)
        .map(|(h, _)| h)
        .collect();
    out.sort();
    Ok(out)
}

#[allow(dead_code)]
fn _silence_cstr(_: &CStr) {}
