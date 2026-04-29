#!/usr/bin/env bash
# neosnatch one-line installer.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/P3X-118/neosnatch/sgc/install.sh | sudo bash
#
# Pulls the latest .deb from the GitHub release matching the host's
# dpkg architecture, installs it, and triggers the first privileged
# snapshot so the banner has data on the next login.

set -euo pipefail

REPO="P3X-118/neosnatch"

if ! command -v dpkg >/dev/null 2>&1; then
    echo "neosnatch currently ships as a Debian package only. Build from source:" >&2
    echo "    git clone https://github.com/${REPO} && cd neosnatch && bash scripts/build-deb.sh" >&2
    exit 1
fi

if [ "$(id -u)" -ne 0 ]; then
    echo "This installer needs to run as root (it calls dpkg). Re-run with sudo." >&2
    exit 1
fi

ARCH=$(dpkg --print-architecture)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

echo "==> resolving latest release for ${ARCH}"
ASSET_URL=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep -E '"browser_download_url"\s*:\s*"[^"]*\.deb"' \
    | grep -E "_${ARCH}\.deb\"" \
    | head -n1 \
    | sed -E 's/.*"(https:[^"]+)".*/\1/')

if [ -z "${ASSET_URL}" ]; then
    echo "No .deb found for arch ${ARCH} on the latest release." >&2
    echo "Available assets:" >&2
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep -E '"browser_download_url"' >&2 || true
    exit 1
fi

echo "==> downloading ${ASSET_URL}"
curl -fsSL "${ASSET_URL}" -o "${TMP}/neosnatch.deb"

echo "==> installing"
dpkg -i "${TMP}/neosnatch.deb" || {
    echo "==> resolving missing deps via apt"
    apt-get update -qq && apt-get install -fy
}

echo
echo "==> first snapshot"
systemctl start neosnatch-snapshot.service || true
sleep 1
if [ -f /var/cache/neosnatch/snapshot.json ]; then
    SCHEMA=$(grep -oE '"schema": *[0-9]+' /var/cache/neosnatch/snapshot.json | head -n1 | grep -oE '[0-9]+' || echo "?")
    echo "    /var/cache/neosnatch/snapshot.json (schema ${SCHEMA}) ready."
fi

echo
echo "Installed. Run 'neosnatch' to see the banner; it will fire on every"
echo "interactive login via /etc/profile.d/neosnatch.sh."
