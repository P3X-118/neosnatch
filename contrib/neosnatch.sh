# /etc/profile.d/neosnatch.sh
# Display sysadmin login banner on interactive shell start.
case $- in
    *i*) command -v neosnatch >/dev/null 2>&1 && neosnatch ;;
esac
