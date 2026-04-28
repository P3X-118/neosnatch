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
    render::print(&cfg, &args, &facts)?;
    Ok(())
}
