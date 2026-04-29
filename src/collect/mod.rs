pub mod os;
pub mod cpu;
pub mod mem;
pub mod disk;
pub mod net;
pub mod uptime;
pub mod sessions;
pub mod systemd;
pub mod ports;
pub mod advisories;
pub mod public_ip;
pub mod packages;
pub mod host;
pub mod gpu;
pub mod shell;
pub mod processes;
pub mod docker;
pub mod sudoers;
pub mod cron;
pub mod encryption;
pub mod services;
pub mod demo;

use crate::cli::Args;
use crate::config::Config;

#[derive(Default, Debug)]
pub struct Facts {
    pub host: Option<String>,
    pub user: Option<String>,
    pub os: Option<os::OsInfo>,
    pub kernel: Option<String>,
    pub kernel_build: Option<String>,
    pub arch: Option<String>,
    pub uptime: Option<uptime::Uptime>,
    pub load: Option<cpu::Load>,
    pub cpu: Option<cpu::CpuInfo>,
    pub mem: Option<mem::MemInfo>,
    pub disks: Vec<disk::DiskInfo>,
    pub interfaces: Vec<net::IfaceInfo>,
    pub public_ip: Option<String>,
    pub sessions: Vec<sessions::Session>,
    pub last_login: Option<sessions::Session>,
    pub failed_units: Vec<String>,
    pub listening_ports: Vec<ports::Listener>,
    pub advisories: Option<advisories::Advisories>,
    pub packages: Option<u64>,
    pub packages_size_kb: Option<u64>,
    pub packages_manager: Option<&'static str>,
    pub packages_manual: Vec<String>,
    pub services_non_default: Vec<services::ServiceUnit>,
    pub encryption: Option<encryption::Encryption>,
    pub sudoers: Vec<sudoers::SudoersRule>,
    pub cron_jobs: Vec<cron::CronJob>,
    #[allow(dead_code)]
    pub anomalous_login: bool,
    pub known_login_hosts: Vec<String>,
    pub host_info: Option<host::HostInfo>,
    pub gpus: Vec<gpu::Gpu>,
    pub shell: Option<shell::ShellInfo>,
    pub docker_networks: docker::NetworkMap,
    pub docker_container_ports: Vec<docker::ContainerPort>,
    /// Age of the privileged snapshot if one was loaded; None if no snapshot.
    pub snapshot_age_secs: Option<u64>,
}

pub async fn gather(cfg: &Config, args: &Args) -> Facts {
    if args.demo { return demo::fixture(); }
    let want_net = !args.offline;
    let s = &cfg.show;

    let snap = crate::snapshot::read(std::path::Path::new(crate::snapshot::DEFAULT_PATH));
    let snapshot_age_secs = snap.as_ref().and_then(crate::snapshot::age_secs);

    let host = hostname();
    let user = current_user();
    let os = if s.os { os::detect().ok() } else { None };
    let (kernel, kernel_build, arch) = uname_full();
    let host_info = if s.model || s.virt { Some(host::detect()) } else { None };
    let gpus = if s.gpu { gpu::list() } else { vec![] };
    let shell_info = if s.shell { shell::detect() } else { None };
    let uptime = if s.uptime { uptime::read().ok() } else { None };
    let load = if s.load { cpu::load().ok() } else { None };
    let cpu = if s.cpu { cpu::info().ok() } else { None };
    let mem = if s.memory { mem::read().ok() } else { None };
    let disks = if s.disk { disk::list().unwrap_or_default() } else { vec![] };
    let interfaces = if s.network { net::list().unwrap_or_default() } else { vec![] };
    let sessions = if s.sessions { sessions::active().unwrap_or_default() } else { vec![] };
    let last_login = if s.last_login { sessions::last().ok().flatten() } else { None };
    let mut listening_ports = if s.listening_ports { ports::list().unwrap_or_default() } else { vec![] };
    if let Some(s) = &snap {
        merge_listener_processes(&mut listening_ports, &s.listeners);
    }
    let pkg_stats = if s.packages { packages::count().await } else { None };
    let packages = pkg_stats.as_ref().map(|s| s.count);
    let packages_size_kb = pkg_stats.as_ref().and_then(|s| s.total_kb);
    let packages_manager = pkg_stats.as_ref().and_then(|s| s.manager);
    let packages_manual = pkg_stats.map(|s| s.manual).unwrap_or_default();
    let services_non_default = services::non_default(&packages_manual);
    let encryption = Some(encryption::detect());

    // Sudoers + cron come exclusively from the privileged snapshot service.
    // Reading cron drop-ins on the login path would only ever show a partial
    // view (root-only crontabs invisible) and split the trust boundary —
    // keep one source of truth.
    let mut sudoers_v: Vec<sudoers::SudoersRule> = Vec::new();
    let mut cron_v: Vec<cron::CronJob> = Vec::new();
    let mut anomalous_login = false;

    let failed_fut = async {
        if s.failed_units { systemd::failed_units().await.unwrap_or_default() } else { vec![] }
    };
    // Advisories are now collected helper-side (see snapshot::generate). Only
    // do a live lookup if we have no snapshot — preserves behavior on hosts
    // without the snapshot service installed.
    let advisories_fut = async {
        let need_live = snap.is_none() && s.advisories;
        if need_live { advisories::check(args.cache_ttl).await } else { None }
    };
    let public_ip_fut = async {
        if s.public_ip && want_net {
            public_ip::fetch(&cfg.network, args.cache_ttl).await
        } else { None }
    };
    let docker_fut = async {
        if s.network { docker::lookup().await } else { docker::NetworkMap::default() }
    };

    let (failed_units, advisories, public_ip, mut docker_networks) =
        tokio::join!(failed_fut, advisories_fut, public_ip_fut, docker_fut);

    // Snapshot fallback: if live lookup gave nothing (e.g. user not in docker
    // group, or `docker` binary unavailable), use snapshot's networks. Same for
    // failed units when systemd D-Bus call was blocked.
    let mut failed_units = failed_units;
    let mut advisories = advisories;
    if let Some(s) = &snap {
        if s.advisories.is_some() { advisories = s.advisories.clone(); }
        if docker_networks.by_bridge.is_empty() && !s.docker_networks.is_empty() {
            docker_networks.by_bridge = s.docker_networks.clone();
        }
        if failed_units.is_empty() && !s.failed_units.is_empty() {
            failed_units = s.failed_units.clone();
        }
        if !s.sudoers.is_empty() {
            sudoers_v = s.sudoers.iter().map(|r| sudoers::SudoersRule {
                source: r.source.clone(),
                principal: r.principal.clone(),
                runas: r.runas.clone(),
                nopasswd: r.nopasswd,
                command: r.command.clone(),
            }).collect();
        }
        if !s.cron_jobs.is_empty() {
            cron_v = s.cron_jobs.iter().map(|c| cron::CronJob {
                source: c.source.clone(),
                schedule: c.schedule.clone(),
                user: c.user.clone(),
                command: c.command.clone(),
            }).collect();
        }
        // Anomaly: a recent login host that snapshot has not yet seen ≥2 times.
        let known: std::collections::HashSet<&str> =
            s.known_login_hosts.iter().map(String::as_str).collect();
        let suspect = |h: &Option<String>| {
            h.as_deref().map(|x| !x.is_empty() && !known.contains(x)).unwrap_or(false)
        };
        if suspect(&last_login.as_ref().and_then(|l| l.host.clone())) {
            anomalous_login = true;
        }
        if sessions.iter().any(|x| suspect(&x.host)) {
            anomalous_login = true;
        }
    }

    Facts {
        host,
        user,
        os,
        kernel,
        kernel_build,
        arch,
        uptime,
        load,
        cpu,
        mem,
        disks,
        interfaces,
        public_ip,
        sessions,
        last_login,
        failed_units,
        listening_ports,
        advisories,
        packages,
        packages_size_kb,
        packages_manager,
        packages_manual,
        services_non_default,
        encryption,
        sudoers: sudoers_v,
        cron_jobs: cron_v,
        anomalous_login,
        known_login_hosts: snap.as_ref().map(|s| s.known_login_hosts.clone()).unwrap_or_default(),
        host_info,
        gpus,
        shell: shell_info,
        docker_networks,
        docker_container_ports: snap.as_ref()
            .map(|s| s.docker_container_ports.clone()).unwrap_or_default(),
        snapshot_age_secs,
    }
}

/// For each live listener missing a process name, fill it in from the snapshot
/// if (proto, port) matches. Live listener list is authoritative — we only
/// borrow process labels.
fn merge_listener_processes(
    live: &mut [ports::Listener],
    snap: &[crate::snapshot::SnapshotListener],
) {
    use std::collections::HashMap;
    let mut by_pp: HashMap<(String, u16), &str> = HashMap::new();
    for s in snap {
        if let Some(p) = &s.process {
            by_pp.insert((s.proto.clone(), s.port), p.as_str());
        }
    }
    for l in live.iter_mut() {
        if l.process.is_some() { continue; }
        if let Some(name) = by_pp.get(&(l.proto.to_string(), l.port)) {
            l.process = Some((*name).to_string());
        }
    }
}

fn hostname() -> Option<String> {
    rustix::system::uname().nodename().to_str().ok().map(str::to_owned)
}

fn current_user() -> Option<String> {
    let uid = nix::unistd::getuid();
    nix::unistd::User::from_uid(uid).ok().flatten().map(|u| u.name)
}

fn uname_full() -> (Option<String>, Option<String>, Option<String>) {
    let u = rustix::system::uname();
    let kernel = u.release().to_str().ok().map(str::to_owned);
    let kernel_build = u.version().to_str().ok().map(str::to_owned);
    let arch = u.machine().to_str().ok().map(str::to_owned);
    (kernel, kernel_build, arch)
}
