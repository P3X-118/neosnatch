#!/bin/bash
# Build a binary .deb for neosnatch using dpkg-deb directly.
# This bypasses debhelper / dpkg-buildpackage so the package can be built on
# any system with a working Rust toolchain and dpkg-deb.
#
# For upstream Debian submission later, switch to `dpkg-buildpackage -us -uc -b`
# after installing debhelper + dh-cargo.

set -euo pipefail

cd "$(dirname "$0")/.."
ROOT=$(pwd)

VERSION=$(awk '/^version *=/ { gsub(/"/, "", $3); print $3; exit }' Cargo.toml)
ARCH=$(dpkg --print-architecture)
PKG="neosnatch_${VERSION}-1_${ARCH}"
STAGE="$ROOT/target/deb/${PKG}"

echo "==> building release binary"
cargo build --release

echo "==> staging at ${STAGE}"
rm -rf "${STAGE}"
mkdir -p "${STAGE}/DEBIAN"
mkdir -p "${STAGE}/usr/bin"
mkdir -p "${STAGE}/etc/profile.d"
mkdir -p "${STAGE}/etc/neosnatch"
mkdir -p "${STAGE}/lib/systemd/system"
mkdir -p "${STAGE}/usr/share/doc/neosnatch"

install -m 755 target/release/neosnatch          "${STAGE}/usr/bin/neosnatch"
install -m 644 contrib/neosnatch.sh              "${STAGE}/etc/profile.d/neosnatch.sh"
install -m 644 contrib/config.example.toml       "${STAGE}/etc/neosnatch/config.toml.example"
install -m 644 debian/neosnatch-snapshot.service "${STAGE}/lib/systemd/system/neosnatch-snapshot.service"
install -m 644 debian/neosnatch-snapshot.timer   "${STAGE}/lib/systemd/system/neosnatch-snapshot.timer"
install -m 644 debian/copyright                  "${STAGE}/usr/share/doc/neosnatch/copyright"

# Compress changelog (Debian policy).
gzip -9n -c debian/changelog > "${STAGE}/usr/share/doc/neosnatch/changelog.Debian.gz"

# DEBIAN/control with computed Installed-Size.
INSTALLED_KB=$(du -sk "${STAGE}" | cut -f1)
{
    awk '/^Source:/ {next} /^Standards-Version:/ {next} /^Build-Depends:/ {next} \
         /^Vcs-/ {next} /^Rules-Requires-Root:/ {next} /^Homepage:/ {print; next} \
         /^Section:/ {print; next} /^Priority:/ {print; next} \
         /^Maintainer:/ {print; next}' debian/control
    echo "Package: neosnatch"
    echo "Version: ${VERSION}-1"
    echo "Architecture: ${ARCH}"
    echo "Depends: libc6"
    echo "Installed-Size: ${INSTALLED_KB}"
    awk '/^Description:/,EOF' debian/control
} > "${STAGE}/DEBIAN/control"

# Maintainer scripts.
install -m 755 debian/postinst "${STAGE}/DEBIAN/postinst"
install -m 755 debian/prerm    "${STAGE}/DEBIAN/prerm"
install -m 755 debian/postrm   "${STAGE}/DEBIAN/postrm"

# conffiles list — files under /etc are tracked as conffiles automatically by
# dpkg-deb when listed here.
cat > "${STAGE}/DEBIAN/conffiles" <<EOF
/etc/profile.d/neosnatch.sh
EOF

echo "==> building deb"
dpkg-deb --root-owner-group --build "${STAGE}" "target/deb/${PKG}.deb"
echo
echo "Done: target/deb/${PKG}.deb"
ls -lh "target/deb/${PKG}.deb"
