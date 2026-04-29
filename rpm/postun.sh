#!/bin/sh
# %postun — runs after uninstall / post-upgrade.
# $1 == 0 means full removal.
set -e

if [ "$1" = "0" ]; then
    rm -rf /etc/systemd/system/neosnatch-snapshot.service.d
    rm -rf /var/cache/neosnatch
    if getent passwd neosnatch >/dev/null; then
        userdel neosnatch >/dev/null 2>&1 || true
    fi
    if [ -d /run/systemd/system ]; then
        systemctl daemon-reload >/dev/null || true
    fi
fi

exit 0
