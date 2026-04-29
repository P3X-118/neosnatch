mod cli;
mod config;
mod collect;
mod render;
mod cache;
mod snapshot;

use anyhow::Result;
use clap::Parser;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    // Restore default SIGPIPE so piping into `head` exits silently instead of
    // panicking on the next println.
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL); }
    let args = cli::Args::parse();
    if let Some(out) = &args.snapshot {
        return snapshot::generate(out).await;
    }
    let cfg = config::load(args.config.as_deref())?;
    let facts = collect::gather(&cfg, &args).await;
    if args.ports { return print_ports_full(&facts); }
    if args.cron  { return print_cron_full(&facts); }
    render::print(&cfg, &args, &facts)?;
    Ok(())
}

fn print_ports_full(facts: &collect::Facts) -> Result<()> {
    use std::net::IpAddr;
    println!("{:<6}  {:>5}  {}", "scope", "port", "service");
    let mut rows: Vec<(bool, u16, String)> = facts.listening_ports.iter().map(|l| {
        let public = match l.addr {
            IpAddr::V4(v4) => !v4.is_loopback(),
            IpAddr::V6(v6) => !v6.is_loopback(),
        };
        (public, l.port, l.process.clone().unwrap_or_else(|| "?".into()))
    }).collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    rows.dedup();
    for (public, port, svc) in rows {
        let scope = if public { "public" } else { "local" };
        println!("{:<6}  {:>5}  {}", scope, port, svc);
    }
    Ok(())
}

fn print_cron_full(facts: &collect::Facts) -> Result<()> {
    println!("{:<32}  {:<14}  {:<12}  {}", "source", "schedule", "user", "command");
    for c in &facts.cron_jobs {
        println!("{:<32}  {:<14}  {:<12}  {}", c.source, c.schedule, c.user, c.command);
    }
    Ok(())
}
