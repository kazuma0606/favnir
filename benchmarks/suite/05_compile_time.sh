#!/usr/bin/env bash
set -e

FAV="${FAV:-./fav/target/release/fav}"
SRC="${BENCH_SRC:-fav/self/compiler.fav}"

# cold compile（キャッシュ削除後）
rm -rf ~/.fav/cache/
t0=$(date +%s%3N)
$FAV build "$SRC" > /dev/null 2>&1 || true
t1=$(date +%s%3N)
echo "compile_cold_ms=$((t1 - t0))"

# incremental（1ファイル変更後）
touch "$SRC"
t2=$(date +%s%3N)
$FAV build "$SRC" > /dev/null 2>&1 || true
t3=$(date +%s%3N)
echo "compile_incremental_ms=$((t3 - t2))"
