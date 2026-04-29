#!/bin/bash
# Build a static-musl tarball for distros without a native package format
# (Alpine, NixOS, Slackware, generic). Bundles the binary, systemd units,
# profile drop-in, an example config, and an installer.
#
# TARGET=x86_64-unknown-linux-musl  bash scripts/build-tarball.sh   # default
# TARGET=aarch64-unknown-linux-musl bash scripts/build-tarball.sh

set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(awk '/^version *=/ { gsub(/"/, "", $3); print $3; exit }' Cargo.toml)
TARGET=${TARGET:-x86_64-unknown-linux-musl}

case "${TARGET}" in
    x86_64-unknown-linux-musl)  ARCH=x86_64 ;;
    aarch64-unknown-linux-musl) ARCH=aarch64 ;;
    *) echo "unsupported target ${TARGET}" >&2; exit 1 ;;
esac

PKG="neosnatch-${VERSION}-${ARCH}-linux-musl"
STAGE="target/tarball/${PKG}"

echo "==> building ${TARGET}"
cargo build --release --target "${TARGET}"

echo "==> staging at ${STAGE}"
rm -rf "${STAGE}"
mkdir -p "${STAGE}/bin" \
         "${STAGE}/share/profile.d" \
         "${STAGE}/share/neosnatch" \
         "${STAGE}/share/systemd"

install -m 755 "target/${TARGET}/release/neosnatch"   "${STAGE}/bin/neosnatch"
install -m 644 contrib/neosnatch.sh                   "${STAGE}/share/profile.d/neosnatch.sh"
install -m 644 contrib/config.example.toml            "${STAGE}/share/neosnatch/config.toml.example"
install -m 644 debian/neosnatch-snapshot.service      "${STAGE}/share/systemd/neosnatch-snapshot.service"
install -m 644 debian/neosnatch-snapshot.timer        "${STAGE}/share/systemd/neosnatch-snapshot.timer"
install -m 644 LICENSE                                "${STAGE}/LICENSE"
install -m 644 README.md                              "${STAGE}/README.md"

cat > "${STAGE}/install.sh" <<'INSTALLER'
#!/bin/sh
# Manual installer for the static-musl tarball. Run as root.
set -e
PREFIX=${PREFIX:-/usr/local}
HERE=$(cd "$(dirname "$0")" && pwd)

install -m 755 "${HERE}/bin/neosnatch"                              "${PREFIX}/bin/neosnatch"
install -d /etc/profile.d /etc/neosnatch /etc/systemd/system /var/cache/neosnatch
install -m 644 "${HERE}/share/profile.d/neosnatch.sh"               /etc/profile.d/neosnatch.sh
install -m 644 "${HERE}/share/neosnatch/config.toml.example"        /etc/neosnatch/config.toml.example
install -m 644 "${HERE}/share/systemd/neosnatch-snapshot.service"   /etc/systemd/system/neosnatch-snapshot.service
install -m 644 "${HERE}/share/systemd/neosnatch-snapshot.timer"     /etc/systemd/system/neosnatch-snapshot.timer

if ! getent passwd neosnatch >/dev/null 2>&1; then
    useradd --system --no-create-home --home-dir /var/cache/neosnatch \
            --shell /sbin/nologin --user-group neosnatch 2>/dev/null \
        || adduser --system --no-create-home --home /var/cache/neosnatch \
                   --shell /sbin/nologin --group neosnatch
fi
chown neosnatch:neosnatch /var/cache/neosnatch
chmod 0755 /var/cache/neosnatch

if [ -d /run/systemd/system ]; then
    systemctl daemon-reload || true
    systemctl enable --now neosnatch-snapshot.timer || true
    systemctl --no-block start neosnatch-snapshot.service || true
fi

echo "neosnatch installed to ${PREFIX}/bin/neosnatch"
INSTALLER
chmod 755 "${STAGE}/install.sh"

echo "==> creating tarball"
mkdir -p target/tarball
( cd target/tarball && tar czf "${PKG}.tar.gz" "${PKG}" )
ls -lh "target/tarball/${PKG}.tar.gz"
