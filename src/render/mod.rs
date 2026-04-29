mod bar;
mod color;
mod format;
mod logo;

use crate::cli::Args;
use crate::collect::{encryption, Facts};
use crate::config::Config;
use anyhow::Result;
use owo_colors::OwoColorize;
use terminal_size::{terminal_size, Width};

const KEY_W: usize = 11;
const BAR_W: usize = 10;
const DEFAULT_COLS: usize = 78;
const MIN_COLS: usize = 60;
const PORT_LIMIT: usize = 12;
const CRON_LIMIT: usize = 10;

pub fn print(cfg: &Config, args: &Args, facts: &Facts) -> Result<()> {
    let cols = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(DEFAULT_COLS)
        .clamp(MIN_COLS, 120);

    if !args.no_logo {
        let host = facts.host.as_deref().unwrap_or("host");
        let lines = logo::render(cfg, args.logo.as_deref(), host)?;
        let max_visible = lines.iter().map(|l| visible_width(l)).max().unwrap_or(0);
        let pad = cols.saturating_sub(max_visible) / 2;
        let pad_str = " ".repeat(pad);
        for line in lines {
            println!("{}{}", pad_str, line);
        }
        println!();
    }

    print_titlebar(facts, cols);
    println!();
    print_identity(facts);
    println!();
    print_resources(cfg, facts, cols);
    println!();
    print_storage(cfg, facts, cols);
    println!();
    print_network(facts, cols);
    println!();
    print_security(facts, cols);
    println!();
    print_schedule(facts, cols);
    println!();
    print_footer(facts, cols);
    Ok(())
}

fn print_titlebar(f: &Facts, cols: usize) {
    let host = f.host.as_deref().unwrap_or("?");
    let user = f.user.as_deref().unwrap_or("?");
    let title = format!(" {user}@{host} · system status ");
    let dashes = cols.saturating_sub(title.chars().count() + 2);
    let left = dashes / 2;
    let right = dashes - left;
    println!(
        "{}{}{}",
        color::frame(&format!("╭─{}", "─".repeat(left))),
        color::header(&title),
        color::frame(&format!("{}─╮", "─".repeat(right))),
    );
}

fn print_section_header(name: &str, cols: usize) {
    let head = format!("▌ {} ", name.to_ascii_uppercase());
    let rule_len = cols.saturating_sub(head.chars().count() + 2);
    println!(
        "  {}{}",
        color::header(&head),
        color::frame(&"─".repeat(rule_len)),
    );
}

fn print_identity(f: &Facts) {
    let label = |s: &str| color::tag(&format!("{:KEY_W$}", s));
    let blank = || color::tag(&format!("{:KEY_W$}", ""));

    if let Some(u) = &f.uptime {
        println!("  {}  {}", label("Uptime"), color::value(&u.pretty()));
    }

    if let Some(os) = &f.os {
        println!("  {}  {}", label("OS"), color::info(&os.pretty_name));
    }
    if let Some(kernel) = &f.kernel {
        let arch = f.arch.as_deref().unwrap_or("");
        let build = f
            .kernel_build
            .as_deref()
            .and_then(|b| b.split_whitespace().next())
            .unwrap_or("");
        let mut tail: Vec<String> = Vec::new();
        if !arch.is_empty() {
            tail.push(arch.to_string());
        }
        if !build.is_empty() {
            tail.push(build.to_string());
        }
        let suffix = if tail.is_empty() {
            String::new()
        } else {
            format!(" {} {}", color::meta("·"), color::meta(&tail.join(" · ")))
        };
        println!("  {}  {}{}", label("Kernel"), color::info(kernel), suffix);
    }
    if let Some(s) = &f.shell {
        let v = match &s.version {
            Some(v) => format!("{} {v}", s.name),
            None => s.name.clone(),
        };
        println!("  {}  {}", label("Shell"), color::info(&v));
    }
    if let Some(host) = &f.host_info {
        let model = host.model.as_deref().unwrap_or("unknown hardware");
        let cleaned = match host.vendor.as_deref() {
            Some(v) => format::strip_vendor(v, model),
            None => model.to_string(),
        };
        let vendor = host.vendor.as_deref().unwrap_or("");
        let line = if vendor.is_empty() {
            cleaned.clone()
        } else {
            format!("{vendor} {cleaned}")
        };
        let virt = if host.virt == "physical" {
            color::meta(&host.virt)
        } else {
            color::warn(&host.virt)
        };
        println!(
            "  {}  {} {}{}{}",
            label("Host"),
            color::info(&line),
            color::meta("("),
            virt,
            color::meta(")")
        );
    }
    if let Some(p) = f.packages {
        let size = match f.packages_size_kb {
            Some(kb) => format!(
                " {} {}",
                color::meta("·"),
                color::info(&format::human_bytes(kb * 1024))
            ),
            None => String::new(),
        };
        let mgr = match f.packages_manager {
            Some(m) => format!(" {} {}", color::meta("·"), color::meta(m)),
            None => String::new(),
        };
        println!(
            "  {}  {}{}{}",
            label("Packages"),
            color::value(&p.to_string()),
            size,
            mgr
        );
    }
    if !f.packages_manual.is_empty() {
        let cols = terminal_size()
            .map(|(Width(w), _)| w as usize)
            .unwrap_or(DEFAULT_COLS)
            .clamp(MIN_COLS, 120);
        let avail = cols.saturating_sub(2 + KEY_W + 2 + 2);
        let joined = f.packages_manual.join(", ");
        let mut lines = wrap_words(&joined, avail);
        let total = f.packages_manual.len();
        if lines.len() > 4 {
            lines.truncate(4);
            if let Some(last) = lines.last_mut() {
                let extra = format!("  +more (apt-mark showmanual: {total} total)");
                let room = avail.saturating_sub(extra.chars().count() + 1);
                if last.chars().count() > room {
                    *last = last.chars().take(room).collect();
                }
                last.push_str(&extra);
            }
        }
        for (i, line) in lines.iter().enumerate() {
            let l = if i == 0 {
                label("Non-Default")
            } else {
                blank()
            };
            println!("  {}  {}", l, color::meta(line));
        }
    }
}

fn print_resources(cfg: &Config, f: &Facts, cols: usize) {
    print_section_header("resources", cols);

    if let Some(c) = &f.cpu {
        let cleaned = format::clean_cpu(&c.model);
        println!(
            "  {}  {} {} {}",
            color::tag(&format!("{:KEY_W$}", "cpu")),
            color::info(&cleaned),
            color::meta("·"),
            color::info(&format!("{}c", c.cores))
        );
    }
    for g in &f.gpus {
        println!(
            "  {}  {}",
            color::tag(&format!("{:KEY_W$}", "gpu")),
            color::info(&format::gpu_label(g))
        );
    }
    if let Some(load) = &f.load {
        let cores = f.cpu.as_ref().map(|c| c.cores).unwrap_or(1);
        let bar = bar::gauge_load(load.one, cores, BAR_W, &cfg.thresholds);
        println!(
            "  {}  {}  {}",
            color::tag(&format!("{:KEY_W$}", "load")),
            bar,
            color::info(&format!(
                "{:.2}  {:.2}  {:.2}",
                load.one, load.five, load.fifteen
            ))
        );
    }
    if let Some(m) = &f.mem {
        let pct = m.used_pct();
        let bar = bar::gauge_pct(pct, BAR_W, cfg.thresholds.mem_warn, cfg.thresholds.mem_crit);
        println!(
            "  {}  {}  {}  {}",
            color::tag(&format!("{:KEY_W$}", "memory")),
            bar,
            color::value(&format!("{:>3}%", pct)),
            color::info(&format!(
                "{} / {}",
                format::human_bytes(m.used_kb() * 1024),
                format::human_bytes(m.total_kb * 1024)
            ))
        );
        if m.swap_total_kb > 0 {
            let p = m.swap_used_pct();
            let bar = bar::gauge_pct(p, BAR_W, cfg.thresholds.mem_warn, cfg.thresholds.mem_crit);
            println!(
                "  {}  {}  {}  {}",
                color::tag(&format!("{:KEY_W$}", "swap")),
                bar,
                color::value(&format!("{:>3}%", p)),
                color::info(&format!(
                    "{} / {}",
                    format::human_bytes(m.swap_used_kb() * 1024),
                    format::human_bytes(m.swap_total_kb * 1024)
                ))
            );
        }
    }
}

fn print_storage(cfg: &Config, f: &Facts, cols: usize) {
    if f.disks.is_empty() {
        return;
    }
    print_section_header("storage", cols);
    let mount_w = f
        .disks
        .iter()
        .map(|d| d.mount.len())
        .max()
        .unwrap_or(0)
        .max(KEY_W);
    let fs_w = f.disks.iter().map(|d| d.fs.len()).max().unwrap_or(0).max(4);

    let enc_lookup = |mount: &str| -> Option<&encryption::Status> {
        f.encryption
            .as_ref()?
            .mounts
            .iter()
            .find(|m| m.mount == mount)
            .map(|m| &m.status)
    };

    for d in &f.disks {
        let pct = d.used_pct();
        let bar = bar::gauge_pct(
            pct,
            BAR_W,
            cfg.thresholds.disk_warn,
            cfg.thresholds.disk_crit,
        );
        let warn_marker = if pct >= cfg.thresholds.disk_crit {
            color::alert("  ⚠")
        } else if pct >= cfg.thresholds.disk_warn {
            color::warn("  ⚠")
        } else {
            String::new()
        };
        let enc = match enc_lookup(&d.mount) {
            Some(encryption::Status::Encrypted(kind)) => color::safe(kind),
            Some(encryption::Status::Unencrypted) => color::alert("unencrypted"),
            Some(encryption::Status::Unknown) => color::meta("unknown"),
            None => color::meta("—"),
        };
        println!(
            "  {}  {}  {}  {} {}  {}{}",
            color::tag(&format!("{:mount_w$}", d.mount)),
            bar,
            color::value(&format!("{:>3}%", pct)),
            color::info(&format!(
                "{} / {}",
                format::human_bytes(d.used_bytes),
                format::human_bytes(d.total_bytes)
            )),
            color::meta(&format!("{:fs_w$}", d.fs)),
            enc,
            warn_marker
        );
    }
}

fn print_network(f: &Facts, cols: usize) {
    print_section_header("network", cols);

    let (primary, bridges) = format::split_network(&f.interfaces, &f.docker_networks);
    // key_w must fit "docker-ports" / "system-ports" (12 chars) so the
    // port-block first row aligns with its continuation rows.
    let key_w = primary
        .iter()
        .map(|i| i.name.len())
        .max()
        .unwrap_or(0)
        .max(KEY_W)
        .max("system-ports".len());

    match &f.public_ip {
        Some(ip) => println!(
            "  {}  {}",
            color::tag(&format!("{:key_w$}", "WAN-IP")),
            color::value(ip)
        ),
        None => println!(
            "  {}  {}",
            color::tag(&format!("{:key_w$}", "WAN-IP")),
            color::warn("unreachable")
        ),
    }
    for ifi in &primary {
        println!(
            "  {}  {}",
            color::tag(&format!("{:key_w$}", ifi.name)),
            color::info(&format::iface_addrs(ifi))
        );
    }

    if !bridges.is_empty() {
        const MAX_VISIBLE: usize = 12;
        let visible = bridges.iter().take(MAX_VISIBLE).collect::<Vec<_>>();
        let name_w = visible
            .iter()
            .map(|b| b.display_name().chars().count())
            .max()
            .unwrap_or(0);

        for (i, b) in visible.iter().enumerate() {
            let label = if i == 0 { "docker" } else { "" };
            let ip = b.ip.map(|i| i.to_string()).unwrap_or_default();
            let name = b.display_name();
            let styled = if b.name.is_some() {
                color::info(&format!("{:name_w$}", name))
            } else {
                color::meta(&format!("{:name_w$}", name))
            };
            println!(
                "  {}  {}  {}",
                color::tag(&format!("{:key_w$}", label)),
                styled,
                color::meta(&ip)
            );
        }
        if bridges.len() > MAX_VISIBLE {
            let extra = bridges.len() - MAX_VISIBLE;
            println!(
                "  {}  {}",
                color::tag(&format!("{:key_w$}", "")),
                color::meta(&format!("+{extra} more"))
            );
        }
    }

    // Port listings: docker-published first, then non-docker system listeners.
    // Column widths shared across both blocks so they align vertically.
    print_port_blocks(f, key_w);
}

fn print_port_blocks(f: &Facts, key_w: usize) {
    use std::collections::HashSet;
    use std::net::IpAddr;

    // Docker rows
    let docker_rows: Vec<DockerRow> = f
        .docker_container_ports
        .iter()
        .map(|p| {
            let mapping = if p.host_port == p.container_port {
                p.host_port.to_string()
            } else {
                format!("{}→{}", p.host_port, p.container_port)
            };
            DockerRow {
                mapping,
                target: p.container.clone(),
                proto_net: format!("{}/{}", p.proto, p.network),
            }
        })
        .collect();

    // System rows (non-docker host listeners)
    let docker_host_ports: HashSet<u16> = f
        .docker_container_ports
        .iter()
        .map(|p| p.host_port)
        .collect();
    let mut sys_rows: Vec<SysRow> = Vec::new();
    for l in &f.listening_ports {
        if docker_host_ports.contains(&l.port) {
            continue;
        }
        let public = match l.addr {
            IpAddr::V4(v4) => !v4.is_loopback(),
            IpAddr::V6(v6) => !v6.is_loopback(),
        };
        let svc = l.process.clone().unwrap_or_else(|| "?".into());
        if sys_rows
            .iter()
            .any(|r| r.public == public && r.port == l.port && r.service == svc)
        {
            continue;
        }
        sys_rows.push(SysRow {
            public,
            port: l.port,
            service: svc,
        });
    }
    sys_rows.sort_by(|a, b| b.public.cmp(&a.public).then(a.port.cmp(&b.port)));

    let sys_total = sys_rows.len();
    if sys_total > PORT_LIMIT {
        sys_rows.truncate(PORT_LIMIT);
    }

    if docker_rows.is_empty() && sys_rows.is_empty() {
        return;
    }

    // Shared column widths
    let scope_w = 6; // "public" / "local "
    let port_w = docker_rows
        .iter()
        .map(|r| r.mapping.len())
        .chain(sys_rows.iter().map(|r| r.port.to_string().len()))
        .max()
        .unwrap_or(5)
        .max(5);
    let target_w = docker_rows
        .iter()
        .map(|r| r.target.len())
        .chain(sys_rows.iter().map(|r| r.service.len()))
        .max()
        .unwrap_or(0)
        .max(8);

    // Render docker block
    for (i, r) in docker_rows.iter().enumerate() {
        let key = if i == 0 { "docker-ports" } else { "" };
        let scope = if i == 0 {
            color::warn("public")
        } else {
            " ".repeat(scope_w)
        };
        println!(
            "  {}  {}  {}  {}  {}",
            color::tag(&format!("{:key_w$}", key)),
            scope,
            color::value(&format!("{:>port_w$}", r.mapping)),
            color::info(&format!("{:target_w$}", r.target)),
            color::meta(&r.proto_net)
        );
    }

    // Render system block
    let mut last_scope: Option<bool> = None;
    let mut first_overall = true;
    for r in &sys_rows {
        let key = if first_overall { "system-ports" } else { "" };
        let scope = if last_scope != Some(r.public) {
            if r.public {
                color::warn("public")
            } else {
                color::safe("local ")
            }
        } else {
            " ".repeat(scope_w)
        };
        println!(
            "  {}  {}  {}  {}",
            color::tag(&format!("{:key_w$}", key)),
            scope,
            color::value(&format!("{:>port_w$}", r.port.to_string())),
            color::info(&format!("{:target_w$}", r.service))
        );
        last_scope = Some(r.public);
        first_overall = false;
    }
    if sys_total > PORT_LIMIT {
        let extra = sys_total - PORT_LIMIT;
        println!(
            "  {}  {}  {}  {}",
            color::tag(&format!("{:key_w$}", "")),
            " ".repeat(scope_w),
            color::meta(&format!("{:>port_w$}", format!("+{extra}"))),
            color::meta("(neosnatch --ports)")
        );
    }
}

struct DockerRow {
    mapping: String,
    target: String,
    proto_net: String,
}
struct SysRow {
    public: bool,
    port: u16,
    service: String,
}

fn print_security(f: &Facts, cols: usize) {
    print_section_header("security", cols);

    if !f.failed_units.is_empty() {
        let names: Vec<&str> = f
            .failed_units
            .iter()
            .map(|s| s.strip_suffix(".service").unwrap_or(s))
            .collect();
        println!(
            "  {}  {}  {}",
            color::tag(&format!("{:KEY_W$}", "failed")),
            color::alert("⚠"),
            color::alert(&names.join(", "))
        );
    } else {
        println!(
            "  {}  {}",
            color::tag(&format!("{:KEY_W$}", "failed")),
            color::safe("none")
        );
    }

    if let Some(adv) = &f.advisories {
        let primary = if adv.critical > 0 {
            color::alert(&format!("{} crit", adv.critical))
        } else if adv.high > 0 {
            color::warn(&format!("{} high", adv.high))
        } else if adv.total > 0 {
            color::meta(&format!("{} pending", adv.total))
        } else {
            color::safe("up to date")
        };
        println!(
            "  {}  {} {} {}",
            color::tag(&format!("{:KEY_W$}", "updates")),
            primary,
            color::meta("·"),
            color::meta(&adv.source)
        );
    } else {
        println!(
            "  {}  {}",
            color::tag(&format!("{:KEY_W$}", "updates")),
            color::meta("no data")
        );
    }

    print_users(f);

    print_sudoers(f, cols);

    print_daemons(f, cols);
}

fn print_daemons(f: &Facts, cols: usize) {
    if f.services_non_default.is_empty() {
        return;
    }
    let avail = cols.saturating_sub(2 + KEY_W + 2 + 2);
    let names: Vec<&str> = f
        .services_non_default
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    let joined = names.join(", ");
    let lines = wrap_words(&joined, avail);
    for (i, line) in lines.iter().enumerate() {
        let key = if i == 0 { "daemons" } else { "" };
        println!(
            "  {}  {}",
            color::tag(&format!("{:KEY_W$}", key)),
            color::meta(line)
        );
    }
}

fn print_users(f: &Facts) {
    use std::collections::HashSet;
    let known: HashSet<&str> = f.known_login_hosts.iter().map(String::as_str).collect();
    let count = f.sessions.len();
    let plural = if count == 1 { "session" } else { "sessions" };
    println!(
        "  {}  {} active",
        color::tag(&format!("{:KEY_W$}", "users")),
        color::info(&format!("{count} {plural}"))
    );

    if !f.sessions.is_empty() {
        let user_w = f
            .sessions
            .iter()
            .map(|s| s.user.len())
            .max()
            .unwrap_or(0)
            .max(6);
        let line_w = f
            .sessions
            .iter()
            .map(|s| s.line.len())
            .max()
            .unwrap_or(0)
            .max(5);
        let host_w = f
            .sessions
            .iter()
            .map(|s| s.host.as_deref().unwrap_or("local").len())
            .max()
            .unwrap_or(0)
            .max(8);
        for s in &f.sessions {
            let host = s.host.as_deref().unwrap_or("local");
            let when = s.when.as_deref().unwrap_or("");
            let anom = s
                .host
                .as_deref()
                .map(|h| !h.is_empty() && !known.contains(h))
                .unwrap_or(false);
            let host_styled = if anom {
                color::alert(&format!("{:host_w$}", host))
            } else {
                color::info(&format!("{:host_w$}", host))
            };
            let mark = if anom {
                color::alert("  ⚠ new IP")
            } else {
                String::new()
            };
            println!(
                "  {}  {}  {}  {}  {}{}",
                color::tag(&format!("{:KEY_W$}", "")),
                color::info(&format!("{:user_w$}", s.user)),
                color::meta(&format!("{:line_w$}", s.line)),
                host_styled,
                color::meta(when),
                mark
            );
        }
    }

    if let Some(s) = &f.last_login {
        let host = s.host.as_deref().unwrap_or("local");
        let when = s.when.as_deref().unwrap_or("?");
        let already_active = f.sessions.iter().any(|x| {
            x.user == s.user
                && x.host.as_deref() == s.host.as_deref()
                && x.when.as_deref() == s.when.as_deref()
        });
        if !already_active {
            let anom = !host.is_empty() && host != "local" && !known.contains(host);
            let row = format!("{} from {} @ {}", s.user, host, when);
            let styled = if anom {
                color::alert(&row)
            } else {
                color::meta(&row)
            };
            let mark = if anom {
                color::alert("  ⚠ new IP")
            } else {
                String::new()
            };
            println!(
                "  {}  {}{}",
                color::tag(&format!("{:KEY_W$}", "last")),
                styled,
                mark
            );
        }
    }
}

fn print_sudoers(f: &Facts, cols: usize) {
    if f.sudoers.is_empty() {
        println!(
            "  {}  {}",
            color::tag(&format!("{:KEY_W$}", "sudoers")),
            color::meta("no data")
        );
        return;
    }
    let label_w = f
        .sudoers
        .iter()
        .map(|r| r.principal.len())
        .max()
        .unwrap_or(0)
        .max(8);
    for (i, r) in f.sudoers.iter().enumerate() {
        let key = if i == 0 { "sudoers" } else { "" };
        let runas = color::meta(&r.runas);
        let cmd_max = cols.saturating_sub(KEY_W + label_w + r.runas.len() + 14);
        let cmd_raw = truncate(&r.command, cmd_max);
        let cmd = if r.command == "ALL" {
            color::alert("ALL")
        } else {
            color::info(&cmd_raw)
        };
        let nopw = if r.nopasswd {
            format!(" {}", color::accent("NOPASSWD"))
        } else {
            String::new()
        };
        println!(
            "  {}  {}  {} {}{}",
            color::tag(&format!("{:KEY_W$}", key)),
            color::info(&format!("{:label_w$}", r.principal)),
            runas,
            cmd,
            nopw
        );
    }
}

fn print_schedule(f: &Facts, cols: usize) {
    print_section_header("schedule", cols);
    if f.cron_jobs.is_empty() {
        println!("  {}", color::meta("no scheduled jobs visible"));
        return;
    }
    let total = f.cron_jobs.len();
    let visible: Vec<&crate::collect::cron::CronJob> =
        f.cron_jobs.iter().take(CRON_LIMIT).collect();
    let src_w = visible
        .iter()
        .map(|c| short_source(&c.source).len())
        .max()
        .unwrap_or(0)
        .max(10);
    let sched_w = visible
        .iter()
        .map(|c| c.schedule.len())
        .max()
        .unwrap_or(0)
        .min(14);
    let user_w = visible
        .iter()
        .map(|c| c.user.len())
        .max()
        .unwrap_or(0)
        .max(4);
    let cmd_w = cols
        .saturating_sub(2 + src_w + 2 + sched_w + 2 + user_w + 2)
        .max(20);
    let prefix_pad = " ".repeat(2 + src_w + 2 + sched_w + 2 + user_w + 2);
    for c in visible {
        let src = short_source(&c.source);
        let lines = wrap_words(&c.command, cmd_w);
        let mut iter = lines.into_iter();
        if let Some(first) = iter.next() {
            println!(
                "  {}  {}  {}  {}",
                color::meta(&format!("{:src_w$}", src)),
                color::tag(&format!("{:sched_w$}", c.schedule)),
                color::info(&format!("{:user_w$}", c.user)),
                color::info(&first)
            );
        }
        for line in iter {
            println!("{}{}", prefix_pad, color::info(&line));
        }
    }
    if total > CRON_LIMIT {
        let extra = total - CRON_LIMIT;
        println!(
            "  {}  {}",
            " ".repeat(src_w),
            color::meta(&format!("+{extra} more (neosnatch --cron)"))
        );
    }
}

fn short_source(s: &str) -> String {
    s.strip_prefix("/etc/")
        .map(str::to_string)
        .unwrap_or_else(|| s.to_string())
}

fn print_footer(f: &Facts, cols: usize) {
    let when = match f.snapshot_age_secs {
        Some(age) => fmt_age(age),
        None => "never".into(),
    };
    let footer = format!(" {} {} ", "Last Report:", when);
    let dashes = cols.saturating_sub(footer.chars().count() + 2);
    let left = dashes / 2;
    let right = dashes - left;
    println!(
        "{}{}{}{}{}",
        color::frame(&format!("╰─{}", "─".repeat(left))),
        color::meta(" Last Report: "),
        color::info(&when),
        color::meta(" "),
        color::frame(&format!("{}─╯", "─".repeat(right))),
    );
}

fn fmt_age(secs: u64) -> String {
    if secs < 90 {
        format!("{secs}s ago")
    } else if secs < 5400 {
        format!("{}m ago", secs / 60)
    } else if secs < 172_800 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86_400)
    }
}

/// Visible character width of a string (ignores CSI escape sequences).
fn visible_width(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut w = 0usize;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() && !((bytes[i] as char).is_ascii_alphabetic()) {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            continue;
        }
        if bytes[i] < 0x80 || (bytes[i] & 0xC0) != 0x80 {
            w += 1;
        }
        i += 1;
    }
    w
}

/// Word-wrap a string into lines of at most `max` chars. Splits on whitespace;
/// if a single token exceeds `max`, it is hard-chunked.
fn wrap_words(s: &str, max: usize) -> Vec<String> {
    if max == 0 {
        return vec![s.to_string()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        let wlen = word.chars().count();
        if wlen > max {
            if !cur.is_empty() {
                lines.push(std::mem::take(&mut cur));
            }
            let mut chunk = String::new();
            for ch in word.chars() {
                if chunk.chars().count() == max {
                    lines.push(std::mem::take(&mut chunk));
                }
                chunk.push(ch);
            }
            cur = chunk;
            continue;
        }
        let needed = if cur.is_empty() {
            wlen
        } else {
            cur.chars().count() + 1 + wlen
        };
        if needed > max {
            lines.push(std::mem::take(&mut cur));
            cur.push_str(word);
        } else {
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(word);
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[allow(dead_code)]
fn truncate(s: &str, max: usize) -> String {
    if max <= 1 {
        return String::new();
    }
    let n = s.chars().count();
    if n <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[allow(dead_code)]
fn _silence_owo(s: &str) -> String {
    s.bold().to_string()
}
