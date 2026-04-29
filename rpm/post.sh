#!/bin/sh
# %post — runs after install / upgrade.
set -e

# System user.
if ! getent passwd neosnatch >/dev/null; then
    useradd --system --no-create-home --home-dir /var/cache/neosnatch \
            --shell /sbin/nologin --user-group neosnatch
fi

# Cache dir.
if [ ! -d /var/cache/neosnatch ]; then
    install -d -o neosnatch -g neosnatch -m 0755 /var/cache/neosnatch
else
    chown neosnatch:neosnatch /var/cache/neosnatch
    chmod 0755 /var/cache/neosnatch
fi

# Conditional supplementary groups.
SUPP=""
if getent group docker >/dev/null 2>&1; then
    usermod -aG docker neosnatch || true
    SUPP="$SUPP docker"
fi
DROPIN_DIR=/etc/systemd/system/neosnatch-snapshot.service.d
DROPIN=$DROPIN_DIR/groups.conf
mkdir -p "$DROPIN_DIR"
if [ -n "$SUPP" ]; then
    {
        echo "[Service]"
        echo "SupplementaryGroups=$SUPP"
    } > "$DROPIN"
else
    rm -f "$DROPIN"
fi

# Reload, enable, start.
if [ -d /run/systemd/system ]; then
    systemctl daemon-reload >/dev/null || true
    systemctl enable --now neosnatch-snapshot.timer >/dev/null || true
    systemctl --no-block start neosnatch-snapshot.service >/dev/null || true
fi

exit 0
