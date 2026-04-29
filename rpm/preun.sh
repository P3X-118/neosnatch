#!/bin/sh
# %preun — runs before uninstall / pre-upgrade.
# $1 == 0 means full removal; $1 == 1 means upgrade.
set -e

if [ "$1" = "0" ] && [ -d /run/systemd/system ]; then
    systemctl --no-block stop neosnatch-snapshot.timer >/dev/null 2>&1 || true
    systemctl disable neosnatch-snapshot.timer >/dev/null 2>&1 || true
fi

exit 0
