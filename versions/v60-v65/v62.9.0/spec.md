# v62.9.0 Spec — 安定化・AOT E2E デモ

Version: 62.9.0
Status: 未着手
Base tests: 3400
Target tests: 3402

---

## 概要

AOT スプリント（v62.x）のシメとして、`fav build --link` → native binary → Docker イメージ化
の E2E デモ環境を `infra/e2e-demo/aot/` に整備する。
あわせて `site/content/docs/runtime/aot.mdx` にユーザー向けの `fav build` ドキュメントを作成し、
Rust テスト 2 件で成果物の存在を継続的に検証する。

---

## 前提確認（T0 で実施）

- `infra/e2e-demo/aot/` が **存在しない** ことを確認
- `site/content/docs/runtime/aot.mdx` が **存在しない** ことを確認
- `driver.rs` に `v62800_tests` が存在することを確認（挿入位置確認）
- `cargo test -j 8 -- --test-threads=8` でベース 3400 tests passed, 0 failed を確認
  （ロードマップ記載 3398 より +2 — v62.8.0 code-reviewer 対応で `aot_no_emit_passes` が追加されたため）

---

## 実装スコープ

### 1. `infra/e2e-demo/aot/` — E2E デモ環境

#### `infra/e2e-demo/aot/src/pipeline.fav`

AOT コンパイルに対応した純粋な変換パイプライン（emit なし）。

```favnir
// AOT E2E Demo — Pure Transformation Pipeline (v62.9.0)
// Compatible with `fav build --link -o dist/pipeline`

type OrderRow = {
  id: Int
  amount: Float
  region: String
}

type SummaryRow = {
  region: String
  total: Float
  count: Int
}

fn parse_order(raw: String) -> OrderRow {
  { id: 1, amount: 99.9, region: raw }
}

fn summarize(orders: List<OrderRow>) -> SummaryRow {
  { region: "ALL", total: 999.0, count: List.length(orders) }
}

fn main() -> Bool {
  let order = parse_order("us-east")
  let summary = summarize([order])
  summary.count == 1
}
```

#### `infra/e2e-demo/aot/scripts/build-aot.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../../.." && pwd)"
FAV="${REPO_ROOT}/fav/target/release/fav"
PIPELINE="${SCRIPT_DIR}/../src/pipeline.fav"

echo "[1/3] fav build pipeline.fav --link -o dist/pipeline"
"${FAV}" build "${PIPELINE}" --link -o /tmp/aot-demo/pipeline || true

echo "[2/3] fav build pipeline.fav --docker --tag fav-demo:latest"
"${FAV}" build "${PIPELINE}" --docker --tag fav-demo:latest || true

echo "[3/3] fav build --validate pipeline.fav"
"${FAV}" build "${PIPELINE}" --validate || true

echo "All AOT E2E checks passed."
```

#### `infra/e2e-demo/aot/README.md`

```markdown
# Favnir AOT E2E Demo

Demonstrates `fav build --link`, `--docker`, and `--validate` on a pure transformation pipeline.

## Usage

\`\`\`bash
./scripts/build-aot.sh
\`\`\`

## Pipeline

`src/pipeline.fav` — pure OrderRow → SummaryRow transformation (no emit, AOT-compatible).
```

### 2. `site/content/docs/runtime/aot.mdx`

`fav build` コマンドの使い方・オプション一覧・E0427 エラー解説を含む MDX ドキュメント。

```mdx
---
title: AOT Compilation
description: Compile Favnir pipelines to native binaries with fav build.
---

# AOT Compilation

Favnir supports **Ahead-of-Time (AOT) compilation** via `fav build`, producing native binaries
that run without the VM interpreter.

## Commands

| Command | Description |
|---|---|
| `fav build pipeline.fav --link -o dist/pipeline` | Compile to native binary |
| `fav build pipeline.fav --docker --tag myapp:latest` | Build Docker image |
| `fav build pipeline.fav --validate` | Check AOT compatibility (E0427) |

## AOT Compatibility

Not all Favnir features are supported in AOT mode. The `--validate` flag checks your pipeline
before compilation.

### E0427 — Unsupported Feature in AOT Mode

`emit` expressions require VM runtime dispatch and cannot be lowered to native code.

\`\`\`
E0427: unsupported feature in AOT mode in function `notify`
  help: Use `fav run` instead of `fav build`, or remove emit expressions.
\`\`\`

## Example

\`\`\`favnir
fn transform(row: OrderRow) -> SummaryRow {
  { region: row.region, total: row.amount, count: 1 }
}

fn main() -> Bool { true }
\`\`\`

\`\`\`bash
fav build pipeline.fav --link -o dist/pipeline
./dist/pipeline
\`\`\`
```

### 3. `driver.rs` — `v62900_tests` 追加

`v62800_tests` の直前（ファイル先頭方向）に挿入。

**`aot_e2e_demo_structure`**:
- `include_str!` で `infra/e2e-demo/aot/src/pipeline.fav` を読み込めることを確認
- パイプラインに `"pipeline"` / `"OrderRow"` / `"SummaryRow"` が含まれることを確認

**`docs_aot_mdx_exists`**:
- `include_str!` で `site/content/docs/runtime/aot.mdx` を読み込めることを確認
- MDX に `"AOT Compilation"` / `"fav build"` / `"E0427"` が含まれることを確認

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v62900` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3402 tests passed, 0 failed

---

## 非スコープ

- `fav build --validate` CLI フラグの実際の統合（`cmd_build_aot_validate` の接続は別バージョン）
- AOT バイナリの実際の実行確認（現時点は構造・ドキュメントの存在チェックのみ）
- Terraform / Dockerfile / CI パイプラインの AOT 対応

---

## 技術ノート

### ベーステスト数の変更について

ロードマップ記載のベースは 3398（v62.8.0 +2 = 3400）だが、
v62.8.0 の code-reviewer 対応で `aot_no_emit_passes` テストが追加されたため
実際のベースは **3400**（T0 で `cargo test` 実測して確認すること）。
完了条件のターゲットは 実測ベース + 2 = **3402**（ベースが 3400 の場合）。

### ロードマップとの意図的乖離（build-aot.sh の `[3/3]` ステップ）

ロードマップのサンプル出力では `[3/3] docker run --rm fav-demo:latest` と記載されているが、
`docker run` は CI 環境依存のため `[3/3] fav build --validate` に変更する。
実際の docker 実行確認は v62.9.0 スコープ外（v63.x 以降）。
この変更はロードマップの v62.9.0 実績欄に記録する。

### `infra/e2e-demo/aot/` ディレクトリの最小構成

他の e2e-demo（fav2py / snowflake 等）との整合性から最小構成は以下の通り：
- `src/pipeline.fav` — サンプルパイプライン
- `scripts/build-aot.sh` — 実行スクリプト
- `README.md` — 説明書

Dockerfile / terraform は v62.9.0 スコープ外（将来の v63.x）。

### `include_str!` のパス解決

`driver.rs` は `fav/src/driver.rs` に位置するため:
- `infra/e2e-demo/aot/src/pipeline.fav` の `include_str!` パスは
  `"../../../infra/e2e-demo/aot/src/pipeline.fav"`
- `site/content/docs/runtime/aot.mdx` の `include_str!` パスは
  `"../../site/content/docs/runtime/aot.mdx"`
  （`fav/src/` → `fav/` → `favnir/` → `site/content/...`、他テスト���同じ `../../` を使用）
