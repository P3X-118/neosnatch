mod color;
mod logo;

use crate::cli::Args;
use crate::collect::Facts;
use crate::config::Config;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn print(cfg: &Config, args: &Args, facts: &Facts) -> Result<()> {
    let logo_lines: Vec<String> = if args.no_logo {
        Vec::new()
    } else {
        logo::render(cfg, args.logo.as_deref()).unwrap_or_default()
    };
    let fact_lines = render_facts(cfg, facts);

    let logo_w = logo_lines.iter().map(|l| visible_width(l)).max().unwrap_or(0);
    let pad = if logo_w == 0 { 0 } else { logo_w + 2 };
    let rows = logo_lines.len().max(fact_lines.len());

    for i in 0..rows {
        let l = logo_lines.get(i).map(String::as_str).unwrap_or("");
        let r = fact_lines.get(i).map(String::as_str).unwrap_or("");
        if pad == 0 {
            println!("{r}");
        } else {
            let lw = visible_width(l);
            let gap = pad.saturating_sub(lw);
            println!("{l}{:gap$}{r}", "", gap = gap);
        }
    }
    Ok(())
}

fn render_facts(cfg: &Config, f: &Facts) -> Vec<String> {
    let mut out = Vec::new();

    if let (Some(user), Some(host)) = (&f.user, &f.host) {
        out.push(format!("{}@{}", user.bold().cyan(), host.bold().cyan()));
        out.push("─".repeat(20).bright_black().to_string());
    }
    if let Some(os) = &f.os {
        let mut v = os.pretty_name.clone();
        if let Some(ver) = &os.version {
            if !v.contains(ver) { v.push_str(&format!(" ({})", ver)); }
        }
        if !os.id.is_empty() && !v.to_ascii_lowercase().contains(&os.id.to_ascii_lowercase()) {
            v.push_str(&format!(" [{}]", os.id));
        }
        out.push(label("OS", &v));
    }
    if let Some(k) = &f.kernel {
        let arch = f.arch.as_deref().unwrap_or("");
        out.push(label("Kernel", &format!("{k} {arch}").trim().to_string()));
    }
    if let Some(u) = &f.uptime {
        out.push(label("Uptime", &u.pretty()));
    }
    if let Some(l) = &f.load {
        let v = format!("{:.2} {:.2} {:.2}", l.one, l.five, l.fifteen);
        out.push(label("Load", &color::load_color(l.one, &cfg.thresholds, &v)));
    }
    if let Some(c) = &f.cpu {
        out.push(label("CPU", &format!("{} ({}c)", c.model, c.cores)));
    }
    if let Some(m) = &f.mem {
        let pct = m.used_pct();
        let v = format!("{} / {} ({}%)",
            human(m.used_kb() * 1024), human(m.total_kb * 1024), pct);
        out.push(label("Memory", &color::pct_color(pct, cfg.thresholds.mem_warn, cfg.thresholds.mem_crit, &v)));
        if m.swap_total_kb > 0 {
            let p = m.swap_used_pct();
            let v = format!("{} / {} ({}%)",
                human(m.swap_used_kb() * 1024), human(m.swap_total_kb * 1024), p);
            out.push(label("Swap", &color::pct_color(p, cfg.thresholds.mem_warn, cfg.thresholds.mem_crit, &v)));
        }
    }
    for d in &f.disks {
        let pct = d.used_pct();
        let v = format!("{} / {} ({}%) {}",
            human(d.used_bytes), human(d.total_bytes), pct, d.fs);
        let key = format!("Disk {}", d.mount);
        out.push(label(&key, &color::pct_color(pct, cfg.thresholds.disk_warn, cfg.thresholds.disk_crit, &v)));
    }
    for ifi in &f.interfaces {
        let addrs: Vec<String> = ifi.addrs.iter().map(|a| a.to_string()).collect();
        out.push(label(&format!("Net {}", ifi.name), &addrs.join(", ")));
    }
    if let Some(ip) = &f.public_ip {
        out.push(label("Public IP", ip));
    }
    if !f.failed_units.is_empty() {
        let v = format!("{} unit(s): {}", f.failed_units.len(), f.failed_units.join(", "));
        out.push(label("⚠ Failed", &v.red().to_string()));
    }
    if let Some(adv) = &f.advisories {
        if adv.total > 0 {
            let v = format!("{} ({} crit, {} high) via {}",
                adv.total, adv.critical, adv.high, adv.source);
            let styled = if adv.critical > 0 { v.red().bold().to_string() }
                         else if adv.high > 0 { v.yellow().to_string() }
                         else { v };
            out.push(label("Advisories", &styled));
        }
    }
    if !f.listening_ports.is_empty() {
        let s: Vec<String> = f.listening_ports.iter()
            .map(|l| {
                let bind = match l.addr {
                    std::net::IpAddr::V4(v4) if v4.is_unspecified() => "*".to_string(),
                    std::net::IpAddr::V6(v6) if v6.is_unspecified() => "*".to_string(),
                    addr => addr.to_string(),
                };
                format!("{}:{}", bind, l.port)
            })
            .collect();
        out.push(label("Listening", &s.join(" ")));
    }
    if !f.sessions.is_empty() {
        let names: Vec<String> = f.sessions.iter()
            .map(|s| match &s.host {
                Some(h) => format!("{}@{} ({})", s.user, h, s.line),
                None => format!("{} ({})", s.user, s.line),
            }).collect();
        out.push(label("Sessions", &format!("{} — {}", f.sessions.len(), names.join(", "))));
    }
    if let Some(last) = &f.last_login {
        let when = last.when.as_deref().unwrap_or("?");
        let host = last.host.as_deref().unwrap_or("local");
        out.push(label("Last login", &format!("{} from {} at {}", last.user, host, when)));
    }
    if let Some(p) = f.packages {
        out.push(label("Packages", &p.to_string()));
    }
    out
}

fn label(k: &str, v: &str) -> String {
    format!("{}: {}", k.bold(), v)
}

fn human(b: u64) -> String {
    const U: [&str; 5] = ["B", "K", "M", "G", "T"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 { v /= 1024.0; i += 1; }
    if v >= 100.0 || i == 0 { format!("{:.0}{}", v, U[i]) }
    else { format!("{:.1}{}", v, U[i]) }
}

fn visible_width(s: &str) -> usize {
    // strip ANSI escape sequences for width calculation
    let mut w = 0usize;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() && !(0x40..=0x7e).contains(&bytes[i]) { i += 1; }
            i += 1;
        } else {
            // approximate: count chars, not graphemes
            i += 1;
            w += 1;
        }
    }
    // recount as chars to handle multibyte
    let stripped = strip_ansi(s);
    stripped.chars().count().max(w.saturating_sub(stripped.len().saturating_sub(stripped.chars().count())))
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for cc in chars.by_ref() {
                if ('@'..='~').contains(&cc) { break; }
            }
        } else {
            out.push(c);
        }
    }
    out
}
