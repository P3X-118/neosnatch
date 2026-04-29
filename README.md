```
─────────────────────────────────────────
            N E O S N A T C H
```

# Not your Daddy's Neofetch.

A Linux login banner for sysadmins. Same baseline as `neofetch` (OS, kernel,
uptime, CPU, memory, disks) — plus everything you actually need to know
when you SSH in: WAN-IP, what's listening on every port (with process
names), docker container publishes, failed systemd units, pending CVE
advisories, full sudoers list, every cron source on the box, all active
sessions, anomalous-IP red flag, manually-installed packages with size,
non-default daemons, and per-mount LUKS status.

Single ~2MB Rust binary, ~25–50 ms cold start. Privileged data is
collected by a hardened systemd helper that runs every 5 minutes; the
login binary itself is unprivileged.

---

## Install

One-liner — auto-detects the host package format and architecture:

```sh
curl -fsSL https://raw.githubusercontent.com/P3X-118/neosnatch/sgc/install.sh | sudo bash
```

Or grab an artifact from the [latest release][latest]:

| Distro family                                  | Architectures   | Artifact                                        |
|------------------------------------------------|-----------------|-------------------------------------------------|
| Debian / Ubuntu / Mint / Pop!\_OS              | amd64, arm64    | `neosnatch_<ver>-1_<arch>.deb`                  |
| Fedora / RHEL / Rocky / Alma / openSUSE        | x86_64, aarch64 | `neosnatch-<ver>-1.<arch>.rpm`                  |
| Alpine / NixOS / Slackware / static glibc-less | x86_64, aarch64 | `neosnatch-<ver>-<arch>-linux-musl.tar.gz`      |

Manual install:

```sh
sudo dpkg -i  neosnatch_*_amd64.deb         # Debian-family
sudo dnf install neosnatch-*.x86_64.rpm     # Red Hat-family
tar xzf neosnatch-*-x86_64-linux-musl.tar.gz && sudo ./neosnatch-*/install.sh
```

Each artifact installs the binary, the systemd timer + service, the
profile drop-in, and creates the dedicated `neosnatch` system user.
SHA256SUMS for every release is attached to the release page.

[latest]: https://github.com/P3X-118/neosnatch/releases/latest

---

## What it shows

| Section   | Surfaced |
|-----------|----------|
| Identity  | Hostname wordmark · uptime · OS · kernel · shell · host model + virt · package count + manager + manually-installed list |
| Resources | CPU model + cores · GPU · load (1/5/15) · memory · swap (gauge bars) |
| Storage   | Per-mount usage % · fs type · LUKS / unencrypted status |
| Network   | WAN-IP · physical interfaces · docker bridge → network names · docker container port publishes · system listening ports with owning process |
| Security  | Failed systemd units · pending advisories · all active sessions w/ user, tty, host, when · last-login + anomalous-IP alert · sudoers (with `NOPASSWD` highlight) · non-default daemons |
| Schedule  | `/etc/crontab`, `/etc/cron.d/*`, `/etc/cron.{hourly,daily,weekly,monthly}/*`, `/etc/anacrontab`, `/var/spool/cron/crontabs/*` |

---

## Architecture

Two pieces:

1. **`neosnatch`** — the unprivileged login binary. Reads `/proc`, `/sys`,
   `statvfs`, `getifaddrs`, `utmp`, etc. directly. No subprocess forks on
   the hot path.
2. **`neosnatch-snapshot.service`** — a systemd timer that fires every
   5 minutes and writes `/var/cache/neosnatch/snapshot.json`. Captures
   the data the unprivileged binary can't see: socket-inode → process
   name, docker network names, sudoers, root-owned crontabs, advisories,
   the rolling known-host set used for anomaly detection.

The helper drops to a dedicated `neosnatch` system user with only
`CAP_DAC_READ_SEARCH + CAP_SYS_PTRACE`. Hardening:
`ProtectSystem=strict`, `ProtectProc=ptraceable`,
`RestrictAddressFamilies=AF_UNIX AF_NETLINK`, `IPAddressDeny=any`,
`KeyringMode=private`, `UMask=0077`, `RestrictFileSystems=@common`,
tightened `SystemCallFilter`, `MemoryDenyWriteExecute=true`. Effectively
a read-only chroot with a single writable carve-out at
`/var/cache/neosnatch`.

The login binary merges live facts with the snapshot. If the helper
isn't installed, neosnatch still works — it just shows what an
unprivileged user can see.

---

## CLI

```
neosnatch                    # render the banner (default)
neosnatch --demo             # render against a synthetic fixture
neosnatch --no-logo          # skip the hostname wordmark
neosnatch --offline          # skip WAN-IP fetch
neosnatch --ports            # full listening-port table (no truncation)
neosnatch --cron             # full cron inventory (no truncation)
```

---

## Build from source

Requires Rust ≥ 1.75.

```sh
cargo build --release
bash scripts/build-deb.sh         # produces target/deb/neosnatch_*.deb
```

---

## Configure

Defaults are sensible. To override, drop a TOML file at
`~/.config/neosnatch/config.toml` (see
[`contrib/config.example.toml`](contrib/config.example.toml) for the
full surface). System-wide config goes at
`/etc/neosnatch/config.toml`.

---

## Uninstall

```sh
sudo apt purge neosnatch
```

`purge` removes the binary, profile drop-in, systemd unit/timer, the
`neosnatch` system user, the cache directory, and the postinst-generated
SupplementaryGroups drop-in.

---

## License

[MIT](LICENSE).
