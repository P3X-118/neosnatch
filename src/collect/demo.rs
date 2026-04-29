//! Synthetic Facts for deterministic visual iteration. Toggled by --demo.
use super::*;
use std::net::{IpAddr, Ipv4Addr};

pub fn fixture() -> Facts {
    Facts {
        host: Some("sandbox".into()),
        user: Some("oneill".into()),
        os: Some(os::OsInfo {
            pretty_name: "Ubuntu 24.04 LTS".into(),
            id: "ubuntu".into(),
            version: Some("24.04".into()),
        }),
        kernel: Some("6.8.0-106-generic".into()),
        kernel_build: Some("#108-Ubuntu SMP PREEMPT_DYNAMIC 2026-03-14".into()),
        arch: Some("x86_64".into()),
        uptime: Some(uptime::Uptime {
            secs: 40 * 86_400 + 14 * 3600 + 23 * 60,
        }),
        load: Some(cpu::Load {
            one: 0.42,
            five: 0.18,
            fifteen: 0.07,
        }),
        cpu: Some(cpu::CpuInfo {
            model: "Intel(R) Xeon(R) CPU E5540 @ 2.53GHz".into(),
            cores: 4,
        }),
        mem: Some(mem::MemInfo {
            total_kb: 32_120_000,
            available_kb: 22_400_000,
            swap_total_kb: 4_194_300,
            swap_free_kb: 4_194_300,
        }),
        disks: vec![
            disk::DiskInfo {
                mount: "/".into(),
                fs: "ext4".into(),
                total_bytes: 105_086_173_184,
                used_bytes: 24_910_000_000,
            },
            disk::DiskInfo {
                mount: "/boot".into(),
                fs: "ext4".into(),
                total_bytes: 2_040_109_465,
                used_bytes: 332_398_592,
            },
            disk::DiskInfo {
                mount: "/var/lib/docker".into(),
                fs: "btrfs".into(),
                total_bytes: 500_000_000_000,
                used_bytes: 437_000_000_000, // 87% — should warn
            },
        ],
        interfaces: vec![
            iface("enp1s0", "10.6.0.54"),
            iface("docker0", "172.17.0.1"),
            iface("br-3930fae8dd8e", "172.20.0.1"),
            iface("br-6f99cbb4772e", "172.21.0.1"),
            iface("br-90cbf5616a9b", "172.23.0.1"),
            iface("br-94ca12fa519b", "172.18.0.1"),
            iface("br-b686cb4c97c4", "172.22.0.1"),
            iface("br-f40626583dd4", "172.19.0.1"),
        ],
        public_ip: Some("198.51.100.42".into()),
        sessions: vec![
            sessions::Session {
                user: "oneill".into(),
                line: "tty1".into(),
                host: None,
                when: Some("2026-04-25 09:14".into()),
            },
            sessions::Session {
                user: "oneill".into(),
                line: "pts/0".into(),
                host: Some("10.6.0.57".into()),
                when: Some("2026-04-27 18:36".into()),
            },
        ],
        last_login: Some(sessions::Session {
            user: "oneill".into(),
            line: "pts/0".into(),
            host: Some("10.6.0.57".into()),
            when: Some("2026-04-27 18:36".into()),
        }),
        failed_units: vec!["fail2ban.service".into(), "snapd.failure.service".into()],
        listening_ports: vec![
            listener("tcp", "0.0.0.0", 22, Some("sshd")),
            listener("tcp", "0.0.0.0", 80, Some("nginx")),
            listener("tcp", "0.0.0.0", 443, Some("nginx")),
            listener("tcp", "0.0.0.0", 3001, Some("node")),
            listener("tcp", "127.0.0.1", 5432, Some("postgres")),
            listener("tcp", "127.0.0.1", 6379, Some("redis-server")),
            listener("tcp", "127.0.0.54", 53, Some("systemd-resolved")),
        ],
        advisories: Some(advisories::Advisories {
            source: "apt".into(),
            critical: 0,
            high: 7,
            total: 23,
        }),
        packages: Some(2147),
        packages_size_kb: Some(8_400_000), // ~8.0G
        packages_manager: Some("apt"),
        packages_manual: vec![
            "build-essential",
            "curl",
            "docker-ce",
            "fail2ban",
            "git",
            "htop",
            "jq",
            "neovim",
            "nginx",
            "postgresql-16",
            "python3-pip",
            "redis",
            "ripgrep",
            "rustc",
            "tmux",
            "ufw",
            "vim",
            "zsh",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        services_non_default: vec![
            services::ServiceUnit {
                name: "docker".into(),
                package: Some("docker-ce".into()),
            },
            services::ServiceUnit {
                name: "fail2ban".into(),
                package: Some("fail2ban".into()),
            },
            services::ServiceUnit {
                name: "nginx".into(),
                package: Some("nginx".into()),
            },
            services::ServiceUnit {
                name: "postgresql".into(),
                package: Some("postgresql-16".into()),
            },
            services::ServiceUnit {
                name: "redis-server".into(),
                package: Some("redis".into()),
            },
        ],
        encryption: Some(encryption::Encryption {
            mounts: vec![
                encryption::MountStatus {
                    mount: "/".into(),
                    status: encryption::Status::Encrypted("LUKS2".into()),
                },
                encryption::MountStatus {
                    mount: "/boot".into(),
                    status: encryption::Status::Unencrypted,
                },
            ],
        }),
        sudoers: vec![
            sudoers::SudoersRule {
                source: "/etc/sudoers".into(),
                principal: "root".into(),
                runas: "(ALL:ALL)".into(),
                nopasswd: false,
                command: "ALL".into(),
            },
            sudoers::SudoersRule {
                source: "/etc/sudoers.d/oneill".into(),
                principal: "oneill".into(),
                runas: "(ALL)".into(),
                nopasswd: true,
                command: "/usr/bin/systemctl".into(),
            },
            sudoers::SudoersRule {
                source: "/etc/sudoers".into(),
                principal: "%sudo".into(),
                runas: "(ALL:ALL)".into(),
                nopasswd: false,
                command: "ALL".into(),
            },
        ],
        cron_jobs: vec![
            cron::CronJob {
                source: "/etc/crontab".into(),
                schedule: "17 * * * *".into(),
                user: "root".into(),
                command: "cd / && run-parts --report /etc/cron.hourly".into(),
            },
            cron::CronJob {
                source: "/etc/cron.d/atop".into(),
                schedule: "*/10 * * * *".into(),
                user: "root".into(),
                command: "/usr/share/atop/atop.daily".into(),
            },
            cron::CronJob {
                source: "/etc/cron.daily".into(),
                schedule: "daily".into(),
                user: "root".into(),
                command: "logrotate".into(),
            },
            cron::CronJob {
                source: "user:oneill".into(),
                schedule: "0 3 * * *".into(),
                user: "oneill".into(),
                command: "/usr/local/bin/backup.sh".into(),
            },
        ],
        anomalous_login: true,
        known_login_hosts: vec!["10.6.0.50".into()],
        host_info: Some(host::HostInfo {
            model: Some("PowerEdge R610".into()),
            vendor: Some("Dell Inc.".into()),
            virt: "physical".into(),
        }),
        gpus: vec![gpu::Gpu {
            vendor: "AMD".into(),
            model: "1002:67df (Polaris 10)".into(),
        }],
        shell: Some(shell::ShellInfo {
            name: "bash".into(),
            path: "/bin/bash".into(),
            version: Some("5.2.21".into()),
        }),
        snapshot_age_secs: Some(127),
        docker_container_ports: vec![
            docker::ContainerPort {
                container: "traefik".into(),
                network: "proxy-net".into(),
                host_port: 80,
                container_port: 80,
                proto: "tcp".into(),
            },
            docker::ContainerPort {
                container: "traefik".into(),
                network: "proxy-net".into(),
                host_port: 443,
                container_port: 443,
                proto: "tcp".into(),
            },
            docker::ContainerPort {
                container: "grafana".into(),
                network: "monitoring".into(),
                host_port: 3000,
                container_port: 3000,
                proto: "tcp".into(),
            },
            docker::ContainerPort {
                container: "vault".into(),
                network: "vault".into(),
                host_port: 8200,
                container_port: 8200,
                proto: "tcp".into(),
            },
        ],
        docker_networks: docker::NetworkMap {
            by_bridge: [
                ("docker0", "bridge"),
                ("br-3930fae8dd8e", "proxy-net"),
                ("br-6f99cbb4772e", "prod-db"),
                ("br-90cbf5616a9b", "monitoring"),
                ("br-94ca12fa519b", "internal"),
                ("br-b686cb4c97c4", "ci-runners"),
                ("br-f40626583dd4", "vault"),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        },
    }
}

fn iface(name: &str, ip: &str) -> net::IfaceInfo {
    net::IfaceInfo {
        name: name.into(),
        addrs: vec![ip.parse().unwrap()],
    }
}

fn listener(proto: &'static str, ip: &str, port: u16, process: Option<&str>) -> ports::Listener {
    let addr: IpAddr = ip.parse().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    ports::Listener {
        proto,
        addr,
        port,
        process: process.map(str::to_owned),
    }
}
