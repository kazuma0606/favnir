#!/usr/bin/env bash
# コールドスタート計測（--precompiled あり/なし）
set -e

FAV="${FAV:-./fav/target/release/fav}"
PROG="${BENCH_PROG:-fav/fav/tmp/hello.fav}"

# フルコンパイル（コールドスタート）
t0=$(date +%s%3N)
$FAV run "$PROG" > /dev/null 2>&1
t1=$(date +%s%3N)
echo "cold_start_full_ms=$((t1 - t0))"

# --precompiled（事前コンパイル済み）
$FAV compile "$PROG" -o /tmp/bench_cold.favc > /dev/null 2>&1
t2=$(date +%s%3N)
$FAV run --precompiled /tmp/bench_cold.favc > /dev/null 2>&1
t3=$(date +%s%3N)
echo "cold_start_precompiled_ms=$((t3 - t2))"
