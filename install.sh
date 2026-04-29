#!/usr/bin/env bash
# neosnatch one-line installer.
#
#   curl -fsSL https://raw.githubusercontent.com/P3X-118/neosnatch/sgc/install.sh | sudo bash
#
# Picks the right artifact for the host:
#   - dpkg  → .deb        (Debian, Ubuntu, Mint, ...)
#   - rpm   → .rpm        (Fedora, RHEL, Rocky, Alma, openSUSE, ...)
#   - else  → static-musl tarball into /usr/local
# Then triggers the first privileged snapshot.

set -euo pipefail

REPO="P3X-118/neosnatch"

if [ "$(id -u)" -ne 0 ]; then
    echo "This installer needs to run as root. Re-run with sudo." >&2
    exit 1
fi

UNAME_M=$(uname -m)
case "${UNAME_M}" in
    x86_64|amd64)   TARBALL_ARCH=x86_64;  DEB_ARCH=amd64;  RPM_ARCH=x86_64  ;;
    aarch64|arm64)  TARBALL_ARCH=aarch64; DEB_ARCH=arm64;  RPM_ARCH=aarch64 ;;
    *) echo "Unsupported architecture: ${UNAME_M}" >&2; exit 1 ;;
esac

# Pick format.
if command -v dpkg >/dev/null 2>&1; then
    FORMAT=deb
    PATTERN="_${DEB_ARCH}\\.deb"
elif command -v rpm >/dev/null 2>&1; then
    FORMAT=rpm
    PATTERN="\\.${RPM_ARCH}\\.rpm"
else
    FORMAT=tarball
    PATTERN="-${TARBALL_ARCH}-linux-musl\\.tar\\.gz"
fi

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

echo "==> resolving latest ${FORMAT} for ${UNAME_M}"
RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")
ASSET_URL=$(echo "${RELEASE_JSON}" \
    | grep -E '"browser_download_url"' \
    | grep -E "${PATTERN}\"" \
    | head -n1 \
    | sed -E 's/.*"(https:[^"]+)".*/\1/')

if [ -z "${ASSET_URL}" ]; then
    echo "No ${FORMAT} asset matching ${PATTERN} in latest release." >&2
    echo "Available assets:" >&2
    echo "${RELEASE_JSON}" | grep -E '"browser_download_url"' >&2 || true
    exit 1
fi

ASSET_NAME=$(basename "${ASSET_URL}")
echo "==> downloading ${ASSET_NAME}"
curl -fsSL "${ASSET_URL}" -o "${TMP}/${ASSET_NAME}"

case "${FORMAT}" in
    deb)
        echo "==> installing"
        dpkg -i "${TMP}/${ASSET_NAME}" || {
            echo "==> resolving missing deps via apt"
            apt-get update -qq && apt-get install -fy
        }
        ;;
    rpm)
        echo "==> installing"
        if command -v dnf >/dev/null 2>&1; then
            dnf install -y "${TMP}/${ASSET_NAME}"
        elif command -v yum >/dev/null 2>&1; then
            yum install -y "${TMP}/${ASSET_NAME}"
        elif command -v zypper >/dev/null 2>&1; then
            zypper --non-interactive install --allow-unsigned-rpm "${TMP}/${ASSET_NAME}"
        else
            rpm -Uvh "${TMP}/${ASSET_NAME}"
        fi
        ;;
    tarball)
        echo "==> extracting"
        tar -C "${TMP}" -xzf "${TMP}/${ASSET_NAME}"
        DIR=$(find "${TMP}" -maxdepth 1 -mindepth 1 -type d | head -n1)
        ( cd "${DIR}" && bash install.sh )
        ;;
esac

echo
echo "==> first snapshot"
systemctl start neosnatch-snapshot.service 2>/dev/null || true
sleep 1
if [ -f /var/cache/neosnatch/snapshot.json ]; then
    SCHEMA=$(grep -oE '"schema": *[0-9]+' /var/cache/neosnatch/snapshot.json | head -n1 | grep -oE '[0-9]+' || echo "?")
    echo "    /var/cache/neosnatch/snapshot.json (schema ${SCHEMA}) ready."
fi

echo
echo "Installed. Run 'neosnatch' to see the banner; it will fire on every"
echo "interactive login via /etc/profile.d/neosnatch.sh."
