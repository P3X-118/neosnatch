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
    pub host_info: Option<host::HostInfo>,
    pub gpus: Vec<gpu::Gpu>,
    pub shell: Option<shell::ShellInfo>,
    pub docker_networks: docker::NetworkMap,
}

pub async fn gather(cfg: &Config, args: &Args) -> Facts {
    if args.demo { return demo::fixture(); }
    let want_net = !args.offline;
    let s = &cfg.show;

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
    let listening_ports = if s.listening_ports { ports::list().unwrap_or_default() } else { vec![] };
    let packages = if s.packages { packages::count().await } else { None };

    let failed_fut = async {
        if s.failed_units { systemd::failed_units().await.unwrap_or_default() } else { vec![] }
    };
    let advisories_fut = async {
        if s.advisories && want_net { advisories::check(args.cache_ttl).await } else { None }
    };
    let public_ip_fut = async {
        if s.public_ip && want_net {
            public_ip::fetch(&cfg.network, args.cache_ttl).await
        } else { None }
    };
    let docker_fut = async {
        if s.network { docker::lookup().await } else { docker::NetworkMap::default() }
    };

    let (failed_units, advisories, public_ip, docker_networks) =
        tokio::join!(failed_fut, advisories_fut, public_ip_fut, docker_fut);

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
        host_info,
        gpus,
        shell: shell_info,
        docker_networks,
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
