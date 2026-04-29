//! Sudoers inventory. /etc/sudoers and /etc/sudoers.d/* are 0440 root:root, so
//! the unprivileged login path will see nothing — collect_all() runs from the
//! root-owned snapshot generator (CAP_DAC_READ_SEARCH) and stores results into
//! the snapshot for everyone to read.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudoersRule {
    pub source: String,
    pub principal: String, // user, %group, or alias
    pub runas: String,     // "(root)" / "(ALL)" / "(ALL : ALL)"
    pub nopasswd: bool,
    pub command: String,
}

pub fn collect_all() -> Vec<SudoersRule> {
    let mut rules = Vec::new();
    if let Ok(s) = fs::read_to_string("/etc/sudoers") {
        parse_file(&s, "/etc/sudoers", &mut rules);
    }
    let dir = Path::new("/etc/sudoers.d");
    if let Ok(rd) = fs::read_dir(dir) {
        for ent in rd.flatten() {
            let path = ent.path();
            if !path.is_file() {
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.is_empty() || name.starts_with('.') || name.ends_with('~') {
                continue;
            }
            if let Ok(s) = fs::read_to_string(&path) {
                parse_file(&s, &path.display().to_string(), &mut rules);
            }
        }
    }
    rules
}

fn parse_file(raw: &str, source: &str, out: &mut Vec<SudoersRule>) {
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if t.starts_with("Defaults")
            || t.starts_with("Cmnd_Alias")
            || t.starts_with("Host_Alias")
            || t.starts_with("User_Alias")
            || t.starts_with("Runas_Alias")
            || t.starts_with("@includedir")
            || t.starts_with("@include")
        {
            continue;
        }
        if let Some(rule) = parse_rule(t, source) {
            out.push(rule);
        }
    }
}

/// Parse "<principal> <hosts>=<spec>". spec is "[(<runas>)] [NOPASSWD:] <cmd>".
fn parse_rule(line: &str, source: &str) -> Option<SudoersRule> {
    let mut parts = line.splitn(2, char::is_whitespace);
    let principal = parts.next()?.to_string();
    let rest = parts.next()?.trim();
    let (_hosts, spec) = rest.split_once('=')?;
    let mut spec = spec.trim().to_string();

    let mut runas = "(ALL)".to_string();
    if spec.starts_with('(') {
        if let Some(end) = spec.find(')') {
            runas = spec[..=end].to_string();
            spec = spec[end + 1..].trim().to_string();
        }
    }
    let mut nopasswd = false;
    for tag in [
        "NOPASSWD:",
        "PASSWD:",
        "SETENV:",
        "NOSETENV:",
        "EXEC:",
        "NOEXEC:",
    ] {
        if let Some(stripped) = spec.strip_prefix(tag) {
            if tag == "NOPASSWD:" {
                nopasswd = true;
            }
            spec = stripped.trim().to_string();
        }
    }
    Some(SudoersRule {
        source: source.to_string(),
        principal,
        runas,
        nopasswd,
        command: spec,
    })
}
