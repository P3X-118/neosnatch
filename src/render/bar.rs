use crate::config::Thresholds;
use owo_colors::OwoColorize;

const FILLED: &str = "▰";
const EMPTY: &str = "▱";

pub fn gauge_pct(pct: u8, width: usize, warn: u8, crit: u8) -> String {
    let pct = pct.min(100);
    let filled = ((pct as usize) * width).div_ceil(100).min(width);
    let empty = width - filled;
    let bar = format!("{}{}", FILLED.repeat(filled), EMPTY.repeat(empty));
    if pct >= crit {
        bar.red().bold().to_string()
    } else if pct >= warn {
        bar.yellow().to_string()
    } else {
        bar.green().to_string()
    }
}

pub fn gauge_load(one: f32, cores: usize, width: usize, t: &Thresholds) -> String {
    // Saturation = load1 / cores. 1.0 = fully loaded.
    let sat = if cores == 0 { one } else { one / cores as f32 };
    let pct = (sat * 100.0).clamp(0.0, 100.0) as u8;
    gauge_pct(
        pct,
        width,
        (t.load_warn * 100.0) as u8,
        (t.load_crit * 100.0) as u8,
    )
}
