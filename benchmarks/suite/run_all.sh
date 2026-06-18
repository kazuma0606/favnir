#!/usr/bin/env bash
# 全ベンチマークスイートを実行し JSON を出力する
# 使用例: bash benchmarks/suite/run_all.sh --format json > benchmarks/latest.json

set -e
SUITE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FAV="${FAV:-./fav/target/release/fav}"
# --format json (default)
FORMAT="json"
for arg in "$@"; do
  case "$arg" in
    --format) FORMAT_NEXT=1 ;;
    *) [[ "$FORMAT_NEXT" == "1" ]] && FORMAT="$arg" && FORMAT_NEXT=0 ;;
  esac
done
VERSION=$(grep '^version' fav/Cargo.toml | head -1 | sed 's/.*= "//;s/"//')
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

declare -A METRICS

run_sh() {
  local script="$1"
  while IFS='=' read -r key val; do
    [[ "$key" =~ ^[a-z_]+$ ]] && METRICS["$key"]="$val"
  done < <(bash "$script" 2>/dev/null)
}

run_fav() {
  local script="$1"
  while IFS='=' read -r key val; do
    [[ "$key" =~ ^[a-z_]+$ ]] && METRICS["$key"]="$val"
  done < <($FAV run "$script" 2>/dev/null)
}

run_sh  "$SUITE_DIR/01_cold_start.sh"
run_fav "$SUITE_DIR/02_csv_10gb.fav"
run_fav "$SUITE_DIR/03_tight_loop.fav"
run_fav "$SUITE_DIR/04_record_transform.fav"
run_sh  "$SUITE_DIR/05_compile_time.sh"
run_fav "$SUITE_DIR/06_duckdb_query.fav"
run_fav "$SUITE_DIR/07_arrow_parquet.fav"
run_fav "$SUITE_DIR/08_concurrent_stages.fav"

# JSON 出力
echo "{"
echo "  \"version\": \"$VERSION\","
echo "  \"timestamp\": \"$TIMESTAMP\","
echo "  \"metrics\": {"
first=true
for key in "${!METRICS[@]}"; do
  [[ "$first" == "true" ]] || echo ","
  printf "    \"%s\": %s" "$key" "${METRICS[$key]}"
  first=false
done
echo ""
echo "  }"
echo "}"
