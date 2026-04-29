use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "neosnatch", version, about = "Not your Daddy's Neofetch")]
pub struct Args {
    /// Path to config file (default: ~/.config/neosnatch/config.toml)
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Skip network-dependent facts (public IP, advisories)
    #[arg(long)]
    pub offline: bool,

    /// Run all collectors including slow ones
    #[arg(long)]
    pub full: bool,

    /// Disable the chafa logo
    #[arg(long)]
    pub no_logo: bool,

    /// (deprecated, no-op) used to override the chafa-rendered logo image
    #[arg(long, hide = true)]
    pub logo: Option<PathBuf>,

    /// Cache TTL in seconds for slow facts
    #[arg(long, default_value_t = 300)]
    pub cache_ttl: u64,

    /// Print computed config and exit
    #[arg(long)]
    pub print_config: bool,

    /// Render with synthetic facts (for visual iteration)
    #[arg(long)]
    pub demo: bool,

    /// Write a privileged snapshot to PATH and exit (run as root via systemd timer).
    /// Captures listeners with process names, docker network names, failed units.
    #[arg(long, value_name = "PATH")]
    pub snapshot: Option<PathBuf>,

    /// Print the full listening-ports list (no truncation) and exit
    #[arg(long)]
    pub ports: bool,

    /// Print the full cron inventory (no truncation) and exit
    #[arg(long)]
    pub cron: bool,
}
