use crate::config::Thresholds;
use owo_colors::OwoColorize;

pub fn pct_color(pct: u8, warn: u8, crit: u8, text: &str) -> String {
    if pct >= crit { text.red().bold().to_string() }
    else if pct >= warn { text.yellow().to_string() }
    else { text.green().to_string() }
}

pub fn load_color(one: f32, t: &Thresholds, text: &str) -> String {
    if one >= t.load_crit { text.red().bold().to_string() }
    else if one >= t.load_warn { text.yellow().to_string() }
    else { text.green().to_string() }
}
