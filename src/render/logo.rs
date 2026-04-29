//! Hostname rendered as a 5-row block-alphabet wordmark. Replaces the old
//! chafa-image / unicorn-ASCII logo path. Hand-rolled glyphs keep the binary
//! lean and let the wordmark match the rest of the Tron-style palette.

use crate::config::Config;
use anyhow::Result;
use std::path::Path;

use super::color;

const ROWS: usize = 5;
const GUTTER: &str = " ";

/// Render `host` as a block wordmark followed by a thin rule. Returns
/// 6 lines (5 glyph rows + 1 rule). `_cfg`/`_override_path` are accepted
/// for backwards compatibility with the old image-logo plumbing; both are
/// ignored now.
pub fn render(_cfg: &Config, _override_path: Option<&Path>, host: &str) -> Result<Vec<String>> {
    Ok(render_wordmark(host))
}

fn render_wordmark(host: &str) -> Vec<String> {
    let upper = host.to_ascii_uppercase();
    let glyphs: Vec<&[&'static str; ROWS]> = upper.chars().map(glyph_for).collect();

    if glyphs.is_empty() {
        return vec![String::new()];
    }

    let mut rows: Vec<String> = (0..ROWS).map(|_| String::new()).collect();
    for (i, g) in glyphs.iter().enumerate() {
        if i > 0 {
            for r in rows.iter_mut() {
                r.push_str(GUTTER);
            }
        }
        for r in 0..ROWS {
            rows[r].push_str(g[r]);
        }
    }

    let total_width = rows[0].chars().count();
    let mut out: Vec<String> = rows.into_iter().map(|r| color::tag(&r)).collect();
    out.push(color::frame(&"─".repeat(total_width)));

    // Product wordmark, centered under the hostname rule.
    let wordmark = "N E O S N A T C H";
    let pad = total_width.saturating_sub(wordmark.chars().count()) / 2;
    out.push(format!("{}{}", " ".repeat(pad), color::header(wordmark)));
    out
}

/// 5×5 block-letter glyphs. Width is uniform per character class (letters
/// are 5 cols wide; digits 5 cols; hyphen 5 cols). Unknown chars render as
/// a 5×5 hollow box.
fn glyph_for(c: char) -> &'static [&'static str; ROWS] {
    match c {
        'A' => &["█████", "█   █", "█████", "█   █", "█   █"],
        'B' => &["████ ", "█   █", "████ ", "█   █", "████ "],
        'C' => &["█████", "█    ", "█    ", "█    ", "█████"],
        'D' => &["████ ", "█   █", "█   █", "█   █", "████ "],
        'E' => &["█████", "█    ", "███  ", "█    ", "█████"],
        'F' => &["█████", "█    ", "███  ", "█    ", "█    "],
        'G' => &["█████", "█    ", "█  ██", "█   █", "█████"],
        'H' => &["█   █", "█   █", "█████", "█   █", "█   █"],
        'I' => &["█████", "  █  ", "  █  ", "  █  ", "█████"],
        'J' => &["█████", "    █", "    █", "█   █", " ███ "],
        'K' => &["█   █", "█  █ ", "███  ", "█  █ ", "█   █"],
        'L' => &["█    ", "█    ", "█    ", "█    ", "█████"],
        'M' => &["█   █", "██ ██", "█ █ █", "█   █", "█   █"],
        'N' => &["█   █", "██  █", "█ █ █", "█  ██", "█   █"],
        'O' => &["█████", "█   █", "█   █", "█   █", "█████"],
        'P' => &["████ ", "█   █", "████ ", "█    ", "█    "],
        'Q' => &["█████", "█   █", "█ █ █", "█  ██", "█████"],
        'R' => &["████ ", "█   █", "████ ", "█  █ ", "█   █"],
        'S' => &["█████", "█    ", "█████", "    █", "█████"],
        'T' => &["█████", "  █  ", "  █  ", "  █  ", "  █  "],
        'U' => &["█   █", "█   █", "█   █", "█   █", "█████"],
        'V' => &["█   █", "█   █", "█   █", " █ █ ", "  █  "],
        'W' => &["█   █", "█   █", "█ █ █", "██ ██", "█   █"],
        'X' => &["█   █", " █ █ ", "  █  ", " █ █ ", "█   █"],
        'Y' => &["█   █", " █ █ ", "  █  ", "  █  ", "  █  "],
        'Z' => &["█████", "   █ ", "  █  ", " █   ", "█████"],
        '0' => &["█████", "█  ██", "█ █ █", "██  █", "█████"],
        '1' => &["  █  ", " ██  ", "  █  ", "  █  ", "█████"],
        '2' => &["█████", "    █", "█████", "█    ", "█████"],
        '3' => &["█████", "    █", " ████", "    █", "█████"],
        '4' => &["█   █", "█   █", "█████", "    █", "    █"],
        '5' => &["█████", "█    ", "█████", "    █", "█████"],
        '6' => &["█████", "█    ", "█████", "█   █", "█████"],
        '7' => &["█████", "    █", "   █ ", "  █  ", " █   "],
        '8' => &["█████", "█   █", "█████", "█   █", "█████"],
        '9' => &["█████", "█   █", "█████", "    █", "█████"],
        '-' => &["     ", "     ", "█████", "     ", "     "],
        '.' => &["     ", "     ", "     ", "     ", "  █  "],
        ' ' => &["     ", "     ", "     ", "     ", "     "],
        _ => &["█████", "█   █", "█   █", "█   █", "█████"],
    }
}
