//! Cron-job inventory across every place cron can hide on a Debian/Linux box:
//!   /etc/crontab                       — system crontab (user field)
//!   /etc/cron.d/*                      — drop-ins (user field)
//!   /etc/cron.{hourly,daily,weekly,monthly}/*  — run-parts directories
//!   /etc/anacrontab                    — anacron schedule
//!   /var/spool/cron/crontabs/*         — per-user crontabs (root-only)
//!
//! All collection runs from the root-owned snapshot generator (the same
//! service that records listening ports). The unprivileged login path
//! consumes only the snapshot — cron is never read off the live filesystem,
//! which keeps the trust boundary tight and the picture consistent.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub source: String,
    pub schedule: String,
    pub user: String,
    pub command: String,
}

fn collect_files() -> Vec<CronJob> {
    let mut jobs = Vec::new();
    if let Ok(s) = fs::read_to_string("/etc/crontab") {
        parse_system(&s, "/etc/crontab", &mut jobs);
    }
    if let Ok(s) = fs::read_to_string("/etc/anacrontab") {
        parse_anacron(&s, "/etc/anacrontab", &mut jobs);
    }
    for entry in read_dir("/etc/cron.d") {
        if let Ok(s) = fs::read_to_string(&entry) {
            parse_system(&s, &entry.display().to_string(), &mut jobs);
        }
    }
    for dir in [
        "/etc/cron.hourly",
        "/etc/cron.daily",
        "/etc/cron.weekly",
        "/etc/cron.monthly",
    ] {
        let schedule = dir.rsplit('.').next().unwrap_or("");
        for entry in read_dir(dir) {
            if let Some(name) = entry.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "0anacron" {
                    continue;
                }
                jobs.push(CronJob {
                    source: dir.to_string(),
                    schedule: schedule.to_string(),
                    user: "root".to_string(),
                    command: name.to_string(),
                });
            }
        }
    }
    jobs
}

pub fn collect_all() -> Vec<CronJob> {
    let mut jobs = collect_files();
    let spool = Path::new("/var/spool/cron/crontabs");
    if spool.is_dir() {
        for entry in read_dir(spool) {
            let user = entry
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            if let Ok(s) = fs::read_to_string(&entry) {
                parse_user(&s, &format!("user:{user}"), &user, &mut jobs);
            }
        }
    }
    jobs
}

fn read_dir<P: AsRef<Path>>(p: P) -> Vec<PathBuf> {
    let Ok(rd) = fs::read_dir(p) else {
        return Vec::new();
    };
    rd.flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect()
}

fn is_skippable(line: &str) -> bool {
    let t = line.trim();
    t.is_empty()
        || t.starts_with('#')
        || t.starts_with("SHELL=")
        || t.starts_with("PATH=")
        || t.starts_with("MAILTO=")
        || t.starts_with("HOME=")
        || t.starts_with("LOGNAME=")
        || t.starts_with("RANDOM_DELAY=")
        || t.starts_with("START_HOURS_RANGE=")
}

fn parse_system(raw: &str, source: &str, out: &mut Vec<CronJob>) {
    for line in raw.lines() {
        if is_skippable(line) {
            continue;
        }
        let (schedule, rest) = match split_schedule(line) {
            Some(v) => v,
            None => continue,
        };
        let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
        if parts.len() != 2 {
            continue;
        }
        out.push(CronJob {
            source: source.to_string(),
            schedule,
            user: parts[0].to_string(),
            command: parts[1].trim().to_string(),
        });
    }
}

fn parse_user(raw: &str, source: &str, user: &str, out: &mut Vec<CronJob>) {
    for line in raw.lines() {
        if is_skippable(line) {
            continue;
        }
        let (schedule, rest) = match split_schedule(line) {
            Some(v) => v,
            None => continue,
        };
        out.push(CronJob {
            source: source.to_string(),
            schedule,
            user: user.to_string(),
            command: rest.trim().to_string(),
        });
    }
}

fn parse_anacron(raw: &str, source: &str, out: &mut Vec<CronJob>) {
    for line in raw.lines() {
        if is_skippable(line) {
            continue;
        }
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 {
            continue;
        }
        out.push(CronJob {
            source: source.to_string(),
            schedule: format!("every {}d", cols[0]),
            user: "root".to_string(),
            command: cols[3..].join(" "),
        });
    }
}

/// Split a crontab line into (schedule, rest). Handles "@daily", "@reboot",
/// or 5 standard fields (m h dom mon dow).
fn split_schedule(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.starts_with('@') {
        let mut it = trimmed.splitn(2, char::is_whitespace);
        let sched = it.next()?.to_string();
        let rest = it.next()?.to_string();
        return Some((sched, rest));
    }
    let cols: Vec<&str> = trimmed.splitn(6, char::is_whitespace).collect();
    if cols.len() != 6 {
        return None;
    }
    Some((cols[..5].join(" "), cols[5].to_string()))
}
