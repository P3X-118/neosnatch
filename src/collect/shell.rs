use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ShellInfo {
    pub name: String, // bash, zsh, fish, ...
    #[allow(dead_code)] // exposed for future renderer use
    pub path: String, // /bin/bash
    pub version: Option<String>,
}

pub fn detect() -> Option<ShellInfo> {
    let path = std::env::var("SHELL")
        .ok()
        .or_else(user_shell_from_passwd)?;
    let name = Path::new(&path).file_name()?.to_string_lossy().to_string();
    let version = probe_version(&name, &path);
    Some(ShellInfo {
        name,
        path,
        version,
    })
}

fn user_shell_from_passwd() -> Option<String> {
    let uid = nix::unistd::getuid();
    nix::unistd::User::from_uid(uid)
        .ok()
        .flatten()
        .map(|u| u.shell.to_string_lossy().into_owned())
}

type VersionParser = fn(&str) -> Option<String>;

fn probe_version(name: &str, path: &str) -> Option<String> {
    // Single fork, ~5 ms. Each shell prints its version on a different stream
    // and in a different format — best-effort parse.
    let (args, parser): (&[&str], VersionParser) = match name {
        "bash" => (&["--version"], parse_bash),
        "zsh" => (&["--version"], parse_zsh),
        "fish" => (&["--version"], parse_fish),
        "dash" => return Some("dash".into()), // no --version
        "ksh" => (&["--version"], parse_ksh),
        "tcsh" => (&["--version"], parse_tcsh),
        _ => return None,
    };
    let out = Command::new(path).args(args).output().ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    parser(&stdout).or_else(|| parser(&stderr))
}

fn parse_bash(s: &str) -> Option<String> {
    // "GNU bash, version 5.2.21(1)-release ..."
    s.lines()
        .next()?
        .split("version ")
        .nth(1)?
        .split_whitespace()
        .next()
        .map(|v| v.split('(').next().unwrap_or(v).to_string())
}

fn parse_zsh(s: &str) -> Option<String> {
    // "zsh 5.9 (x86_64-...)"
    s.split_whitespace().nth(1).map(str::to_owned)
}

fn parse_fish(s: &str) -> Option<String> {
    // "fish, version 3.7.1"
    s.split("version ")
        .nth(1)?
        .split_whitespace()
        .next()
        .map(str::to_owned)
}

fn parse_ksh(s: &str) -> Option<String> {
    s.lines().next().map(|l| l.trim().to_string())
}

fn parse_tcsh(s: &str) -> Option<String> {
    // "tcsh 6.24.07 (Astron) ..."
    s.split_whitespace().nth(1).map(str::to_owned)
}
