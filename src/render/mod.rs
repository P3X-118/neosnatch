mod bar;
mod color;
mod format;
mod logo;

use crate::cli::Args;
use crate::collect::Facts;
use crate::config::Config;
use anyhow::Result;
use owo_colors::OwoColorize;
use terminal_size::{terminal_size, Width};

const KEY_W: usize = 9;
const BAR_W: usize = 10;
const DEFAULT_COLS: usize = 78;
const MIN_COLS: usize = 60;

pub fn print(cfg: &Config, args: &Args, facts: &Facts) -> Result<()> {
    let cols = terminal_size().map(|(Width(w), _)| w as usize)
        .unwrap_or(DEFAULT_COLS).clamp(MIN_COLS, 120);

    if !args.no_logo {
        for line in logo::render(cfg, args.logo.as_deref())? {
            println!("{line}");
        }
        println!();
    }

    print_titlebar(facts, cols);
    println!();
    print_identity(facts);
    println!();
    print_resources(cfg, facts);
    println!();
    print_storage(cfg, facts);
    println!();
    print_network(facts);
    println!();
    print_security(facts);
    println!();
    print_footer(facts, cols);
    Ok(())
}

fn print_titlebar(f: &Facts, cols: usize) {
    let host = f.host.as_deref().unwrap_or("?");
    let user = f.user.as_deref().unwrap_or("?");
    let title = format!(" neosnatch · {user}@{host} · system status ");
    let dashes = cols.saturating_sub(title.chars().count() + 2);
    let left = dashes / 2;
    let right = dashes - left;
    println!("{}{}{}",
        format!("╭─{}", "─".repeat(left)).bright_black(),
        title.bold().cyan(),
        format!("{}─╮", "─".repeat(right)).bright_black(),
    );
}

fn print_section_header(name: &str, cols: usize) {
    let head = format!("▌ {} ", name.to_ascii_uppercase());
    let rule_len = cols.saturating_sub(head.chars().count() + 2);
    println!("  {}{}",
        head.bold().cyan(),
        "─".repeat(rule_len).bright_black(),
    );
}

fn print_identity(f: &Facts) {
    let user = f.user.as_deref().unwrap_or("?");
    let host = f.host.as_deref().unwrap_or("?");
    let shell = f.shell.as_ref().map(|s| match &s.version {
        Some(v) => format!("{} {v}", s.name),
        None => s.name.clone(),
    }).unwrap_or_default();
    println!("  {}  {}",
        format!("{user}@{host}").bold().cyan(),
        if shell.is_empty() { String::new() } else { format!("· {shell}").bright_black().to_string() });

    if let Some(os) = &f.os {
        let kernel = f.kernel.as_deref().unwrap_or("");
        let arch = f.arch.as_deref().unwrap_or("");
        let parts: Vec<&str> = [os.pretty_name.as_str(), kernel, arch]
            .iter().filter(|s| !s.is_empty()).copied().collect();
        println!("  {}", parts.join(" · "));
    }
    if let Some(build) = &f.kernel_build {
        // First whitespace-delimited token is e.g. "#110-Ubuntu" — enough at a glance.
        let short = build.split_whitespace().next().unwrap_or(build);
        println!("  {}", short.bright_black());
    }

    if let Some(host) = &f.host_info {
        let model = host.model.as_deref().unwrap_or("unknown hardware");
        let cleaned = match host.vendor.as_deref() {
            Some(v) => format::strip_vendor(v, model),
            None => model.to_string(),
        };
        let vendor = host.vendor.as_deref().unwrap_or("");
        let line = if vendor.is_empty() { cleaned.clone() } else { format!("{vendor} {cleaned}") };
        let virt = if host.virt == "physical" {
            host.virt.to_string()
        } else {
            host.virt.yellow().to_string()
        };
        println!("  {} ({virt})", line);
    }
}

fn print_resources(cfg: &Config, f: &Facts) {
    let cols = terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(DEFAULT_COLS);
    print_section_header("resources", cols);

    if let Some(load) = &f.load {
        let cores = f.cpu.as_ref().map(|c| c.cores).unwrap_or(1);
        let bar = bar::gauge_load(load.one, cores, BAR_W, &cfg.thresholds);
        println!("  {:KEY_W$}  {}  {:.2}  {:.2}  {:.2}",
            "load", bar, load.one, load.five, load.fifteen);
    }
    if let Some(c) = &f.cpu {
        let cleaned = format::clean_cpu(&c.model);
        println!("  {:KEY_W$}  {} · {}c", "cpu", cleaned, c.cores);
    }
    for g in &f.gpus {
        println!("  {:KEY_W$}  {}", "gpu", format::gpu_label(g));
    }
    if let Some(m) = &f.mem {
        let pct = m.used_pct();
        let bar = bar::gauge_pct(pct, BAR_W, cfg.thresholds.mem_warn, cfg.thresholds.mem_crit);
        println!("  {:KEY_W$}  {}  {:>3}%  {} / {}",
            "memory", bar, pct,
            format::human_bytes(m.used_kb() * 1024),
            format::human_bytes(m.total_kb * 1024));
        if m.swap_total_kb > 0 {
            let p = m.swap_used_pct();
            let bar = bar::gauge_pct(p, BAR_W, cfg.thresholds.mem_warn, cfg.thresholds.mem_crit);
            println!("  {:KEY_W$}  {}  {:>3}%  {} / {}",
                "swap", bar, p,
                format::human_bytes(m.swap_used_kb() * 1024),
                format::human_bytes(m.swap_total_kb * 1024));
        }
    }
}

fn print_storage(cfg: &Config, f: &Facts) {
    if f.disks.is_empty() { return; }
    let cols = terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(DEFAULT_COLS);
    print_section_header("storage", cols);
    let mount_w = f.disks.iter().map(|d| d.mount.len()).max().unwrap_or(0).max(8);
    for d in &f.disks {
        let pct = d.used_pct();
        let bar = bar::gauge_pct(pct, BAR_W, cfg.thresholds.disk_warn, cfg.thresholds.disk_crit);
        let warn_marker = if pct >= cfg.thresholds.disk_crit { "  ⚠".red().bold().to_string() }
            else if pct >= cfg.thresholds.disk_warn { "  ⚠".yellow().to_string() }
            else { String::new() };
        println!("  {:mount_w$}  {}  {:>3}%  {} / {} {}{}",
            d.mount, bar, pct,
            format::human_bytes(d.used_bytes),
            format::human_bytes(d.total_bytes),
            d.fs.bright_black(),
            warn_marker);
    }
}

fn print_network(f: &Facts) {
    let cols = terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(DEFAULT_COLS);
    print_section_header("network", cols);

    let (primary, bridges) = format::split_network(&f.interfaces, &f.docker_networks);
    let key_w = primary.iter().map(|i| i.name.len()).max().unwrap_or(0).max(KEY_W);

    for ifi in &primary {
        println!("  {:key_w$}  {}", ifi.name, format::iface_addrs(ifi));
    }
    if let Some(ip) = &f.public_ip {
        println!("  {:key_w$}  {}", "public", ip);
    }

    if !bridges.is_empty() {
        const MAX_VISIBLE: usize = 12;
        let visible = bridges.iter().take(MAX_VISIBLE).collect::<Vec<_>>();
        let name_w = visible.iter()
            .map(|b| b.display_name().chars().count())
            .max().unwrap_or(0);

        for (i, b) in visible.iter().enumerate() {
            let label = if i == 0 { "docker" } else { "" };
            let ip = b.ip.map(|i| i.to_string()).unwrap_or_default();
            let name = b.display_name();
            let styled = if b.name.is_some() {
                format!("{:name_w$}", name)
            } else {
                format!("{:name_w$}", name).bright_black().to_string()
            };
            println!("  {:key_w$}  {}  {}", label, styled, ip.bright_black());
        }
        if bridges.len() > MAX_VISIBLE {
            let extra = bridges.len() - MAX_VISIBLE;
            println!("  {:key_w$}  {}", "", format!("+{extra} more").bright_black());
        }
    }
}

fn print_security(f: &Facts) {
    let cols = terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(DEFAULT_COLS);
    print_section_header("security", cols);

    let (pub_svcs, loc_svcs) = format::group_ports_by_service(&f.listening_ports);
    let svc_w = pub_svcs.iter().chain(loc_svcs.iter())
        .map(|s| s.name.chars().count()).max().unwrap_or(0).max(8);

    let mut first_overall = true;
    for (i, sp) in pub_svcs.iter().enumerate() {
        let key = if first_overall { "ports" } else { "" };
        let scope = if i == 0 { "public".yellow().to_string() } else { "      ".to_string() };
        println!("  {:KEY_W$}  {}  {:svc_w$}  {}",
            key, scope, sp.name, format::fmt_ports(&sp.ports).bright_black());
        first_overall = false;
    }
    for (i, sp) in loc_svcs.iter().enumerate() {
        let key = if first_overall { "ports" } else { "" };
        let scope = if i == 0 { "local ".green().to_string() } else { "      ".to_string() };
        println!("  {:KEY_W$}  {}  {:svc_w$}  {}",
            key, scope, sp.name, format::fmt_ports(&sp.ports).bright_black());
        first_overall = false;
    }

    if !f.failed_units.is_empty() {
        let names: Vec<&str> = f.failed_units.iter()
            .map(|s| s.strip_suffix(".service").unwrap_or(s)).collect();
        println!("  {:KEY_W$}  {}  {}",
            "failed", "⚠".red().bold(), names.join(", ").red());
    } else {
        println!("  {:KEY_W$}  {}",
            "failed", "none".green());
    }

    if let Some(adv) = &f.advisories {
        let primary = if adv.critical > 0 {
            format!("{} crit", adv.critical).red().bold().to_string()
        } else if adv.high > 0 {
            format!("{} high", adv.high).yellow().to_string()
        } else if adv.total > 0 {
            format!("{} pending", adv.total).bright_black().to_string()
        } else {
            "up to date".green().to_string()
        };
        println!("  {:KEY_W$}  {} · {}",
            "updates", primary, adv.source.bright_black());
    } else {
        println!("  {:KEY_W$}  {}",
            "updates", "no data".bright_black());
    }

    let count = f.sessions.len();
    let last = f.last_login.as_ref()
        .map(|s| {
            let host = s.host.as_deref().unwrap_or("local");
            let when = s.when.as_deref().unwrap_or("?");
            format!("last: {} from {} @ {}", s.user, host, when)
        })
        .unwrap_or_default();
    let plural = if count == 1 { "session" } else { "sessions" };
    println!("  {:KEY_W$}  {} active   {}",
        "users", format!("{count} {plural}"), last.bright_black());
}

fn print_footer(f: &Facts, cols: usize) {
    let mut parts: Vec<String> = Vec::new();
    if let Some(u) = &f.uptime {
        parts.push(format!("uptime {}", u.pretty()));
    }
    if let Some(p) = f.packages {
        parts.push(format!("{p} packages"));
    }
    if let Some(age) = f.snapshot_age_secs {
        parts.push(format!("snapshot {}", fmt_age(age)));
    }
    let footer = if parts.is_empty() { String::new() } else { format!(" {} ", parts.join(" · ")) };
    let dashes = cols.saturating_sub(footer.chars().count() + 2);
    let left = dashes / 2;
    let right = dashes - left;
    println!("{}{}{}",
        format!("╰─{}", "─".repeat(left)).bright_black(),
        footer.bright_black(),
        format!("{}─╯", "─".repeat(right)).bright_black(),
    );
}

fn fmt_age(secs: u64) -> String {
    if secs < 90 { format!("{secs}s ago") }
    else if secs < 5400 { format!("{}m ago", secs / 60) }
    else if secs < 172_800 { format!("{}h ago", secs / 3600) }
    else { format!("{}d ago", secs / 86_400) }
}
