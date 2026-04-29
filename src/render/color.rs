//! Tron-style palette. One central place so every section reaches for the same
//! handful of roles and we don't drift toward an ANSI rainbow.
//!
//! Roles:
//!   frame   — borders / rules                 cyan
//!   header  — section titles                  bold bright_cyan
//!   tag     — small leading labels (OS:)      bright_cyan
//!   value   — primary data                    bright_white bold
//!   info    — secondary data                  bright_white
//!   meta    — supporting metadata             soft cyan-grey (truecolor)
//!   safe    — green ok-state
//!   warn    — yellow advisory
//!   alert   — red critical / anomaly          bold red
//!   accent  — orange-ish electric highlight   bright_yellow

use crate::config::Thresholds;
use owo_colors::OwoColorize;

const META_R: u8 = 110;
const META_G: u8 = 165;
const META_B: u8 = 185;

pub fn frame(s: &str)   -> String { s.cyan().to_string() }
pub fn header(s: &str)  -> String { s.bright_cyan().bold().to_string() }
pub fn tag(s: &str)     -> String { s.bright_cyan().to_string() }
pub fn value(s: &str)   -> String { s.bright_white().bold().to_string() }
pub fn info(s: &str)    -> String { s.bright_white().to_string() }
pub fn meta(s: &str)    -> String { s.truecolor(META_R, META_G, META_B).to_string() }
pub fn safe(s: &str)    -> String { s.green().to_string() }
pub fn warn(s: &str)    -> String { s.yellow().to_string() }
pub fn alert(s: &str)   -> String { s.bright_red().bold().to_string() }
pub fn accent(s: &str)  -> String { s.bright_yellow().to_string() }

#[allow(dead_code)]
pub fn pct_color(pct: u8, warn_t: u8, crit: u8, text: &str) -> String {
    if pct >= crit { alert(text) }
    else if pct >= warn_t { warn(text) }
    else { safe(text) }
}

#[allow(dead_code)]
pub fn load_color(one: f32, t: &Thresholds, text: &str) -> String {
    if one >= t.load_crit { alert(text) }
    else if one >= t.load_warn { warn(text) }
    else { safe(text) }
}
