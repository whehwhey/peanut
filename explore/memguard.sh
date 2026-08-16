#!/bin/bash
# SYSTEM-LEVEL GUARD (layer 3).  Independent of any Python runner.
#   ./explore/memguard.sh            run in foreground
#   ./explore/memguard.sh --daemon   start detached, pidfile results/memguard.pid
# Every 2 s: kill any engine process over $AM_KILL_MB RSS (default 3072), and if the
# kernel reports memory pressure >= warn (2) or free RAM < $AM_FLOOR_MB (default 4096),
# kill the largest engine.  Logs to results/memguard.log.
# The engine binary was renamed automatheus -> peanut on 2026-08-17; both names are
# matched, because old checkouts, the compatibility symlink and other sessions may still
# be running the previous name.
KILL_MB=${AM_KILL_MB:-3072}; FLOOR_MB=${AM_FLOOR_MB:-4096}
# Under pressure only engines ABOVE this size are candidates.  Killing a 12 MB engine
# does not relieve pressure caused by some other process; it just makes the repo
# unusable whenever anything else on the machine is big.  The hard KILL_MB ceiling
# above is unchanged, so the actual protection is untouched.
PRESSURE_MIN_MB=${AM_PRESSURE_MIN_MB:-384}
cd "$(dirname "$0")/.."
if [ "$1" = "--daemon" ]; then
  if [ -f results/memguard.pid ] && kill -0 "$(cat results/memguard.pid)" 2>/dev/null; then echo "memguard already running"; exit 0; fi
  nohup "$0" >> results/memguard.log 2>&1 &
  echo $! > results/memguard.pid; echo "memguard started pid $!"; exit 0
fi
free_mb() { vm_stat | awk '/page size of/{ps=$8} /Pages free/{f=$3} /Pages inactive/{i=$3} /Pages speculative/{s=$3} END{gsub(/\./,"",f);gsub(/\./,"",i);gsub(/\./,"",s); print (f+i+s)*ps/1048576}' | cut -d. -f1; }
while true; do
  ps -axo pid,rss,comm | awk -v k=$((KILL_MB*1024)) '$3 ~ /(peanut|automatheus)$/ && $2>k {print $1, $2}' | while read pid rss; do
    echo "$(date '+%F %T') kill pid $pid rss $((rss/1024))MB > ${KILL_MB}MB"; kill -9 "$pid" 2>/dev/null; done
  P=$(sysctl -n kern.memorystatus_vm_pressure_level 2>/dev/null || echo 1); F=$(free_mb)
  if [ "$P" -ge 2 ] || [ "$F" -lt "$FLOOR_MB" ]; then
    big=$(ps -axo pid,rss,comm | awk -v m=$((PRESSURE_MIN_MB*1024)) '$3 ~ /(peanut|automatheus)$/ && $2>m {print $2, $1}' | sort -rn | head -1)
    if [ -n "$big" ]; then set -- $big; echo "$(date '+%F %T') PRESSURE level=$P free=${F}MB: kill biggest pid $2 ($(( $1/1024 ))MB)"; kill -9 "$2" 2>/dev/null; fi
  fi
  sleep 2
done
