mod cli;
mod config;
mod collect;
mod render;
mod cache;

use anyhow::Result;
use clap::Parser;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    let args = cli::Args::parse();
    let cfg = config::load(args.config.as_deref())?;
    let facts = collect::gather(&cfg, &args).await;
    render::print(&cfg, &args, &facts)?;
    Ok(())
}
