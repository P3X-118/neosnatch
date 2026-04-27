# neosnatch

Login-banner sysadmin stats for Linux terminals. A focused Rust rewrite inspired by neofetch — drops the distro-flex eye-candy in favor of fast, useful information for operators.

## What it shows

- Host / user / OS / kernel / uptime
- Load avg, CPU, memory, swap (thresholded coloring)
- Real filesystems: usage % per mount
- Network interfaces + public IP (via `https://ip.sgc.ai`)
- Failed `systemd` units
- Listening ports summary
- Active sessions, last login
- Pending security advisories (per-distro)

## Design

- **Linux only.** Reads `/proc`, `/sys`, `statvfs`, `getifaddrs`, D-Bus directly. Minimal subprocess fork.
- **Fast.** Target < 50 ms cold start with default facts. Network/advisory checks are async + cached on disk (`~/.cache/neosnatch/`).
- **Single static binary** (`x86_64-unknown-linux-musl`).
- **Config:** typed TOML at `~/.config/neosnatch/config.toml` (see `contrib/config.example.toml`).
- **Logo:** one image, rendered via `chafa` (must be installed). Override with `--logo PATH`.

## Build

```sh
cargo build --release
# or static:
make release
```

## Install

```sh
sudo make install
```

This drops `/etc/profile.d/neosnatch.sh` so the banner shows on interactive login.

## Status

Alpha. Collectors implemented now: os, kernel, uptime, cpu, load, mem, disk, network interfaces, public IP, two-column renderer with chafa logo. Stubbed for next pass: utmp sessions, systemd D-Bus, listening-port `/proc/net/tcp` parser, distro-specific advisory adapters, package counts.
