# v70.1.0 — Backlog Blitz（積み残し一掃）

Date: 2026-08-08
Status: 計画中

ロードマップ: [roadmap-v70.1-v71.0.md](../../roadmap/roadmap-v70.1-v71.0.md)

---

## Background

v65〜v70 のスプリントで積み上がった技術的負債を一括解消するバージョン。
特に `bench.yml` CI の長期的な障害（compare / regression ステップが
`continue-on-error: true` で無条件スキップされている問題）を根治する。

根本原因は `compiler.fav` が複数パラメータ関数 (`fn f(ctx: AppCtx, data: T)`) を
パースできないこと。v70.0 で `benchmarks/compare.fav` を `ctx: AppCtx` 構文に
移行済みだが、compiler.fav のパースエラーにより CI 上で実行できず、暫定的に
`continue-on-error: true` が設定されたままになっている。

**注意**: `benchmarks/compare.fav` のソースコード自体は v70.0 で既に `ctx: AppCtx` 構文に
移行完了済みであり、v70.1.0 での追加ソース変更は不要。`parse_fn_params` の修正が完了すれば
そのまま動作するようになる。

---

## Goals

1. `compiler.fav` の `parse_fn_params` を修正し、複数パラメータ関数を正しく解析する
2. `bench.yml` から `continue-on-error: true` を除去し、CI を完全グリーンにする
3. `versions/current.md` を v70.0.0 完了・v70.1.0 進行中に更新する

---

## 積み残し詳細

| 項目 | 症状 | 修正方針 |
|---|---|---|
| `compiler.fav` multi-param ctx 未対応 | `fn f(ctx: AppCtx, data: T)` のパースが失敗 | `parse_fn_params` に `,` を挟んだ複数パラメータ対応を追加 |
| `bench.yml` の `continue-on-error` 暫定対応 | Compare / Regression ステップが無条件スキップ | compiler.fav 修正後に外す |
| `versions/current.md` 旧情報 | "次に切る版" が v66.9.0 のまま | v70.0.0 完了・v70.1.0 進行中に同期 |

---

## 修正仕様

### 1. `compiler.fav` — `parse_fn_params` 複数パラメータ対応

**現状（問題）:**
```
fn write_results_md(ctx: AppCtx, data: JsonValue) -> Result<Unit, String>
```
この形式をパースしようとすると、compiler.fav の `parse_fn_params` が
最初のパラメータ (`ctx: AppCtx`) を読んだ後に `,` をトークンとして
認識できずにパニックまたは誤ったパースを行う。

**修正後（期待動作）:**
```favnir
// 複数パラメータ関数が正しくパース・コンパイルされる
fn write_results_md(ctx: AppCtx, data: JsonValue) -> Result<Unit, String> {
    bind version <- Json.get_string(data, "version")
    ctx.io.write_file_raw("results.md", f"# v{version}")
}

fn main(ctx: AppCtx) -> Result<Unit, String> {
    bind args <- ctx.io.argv()
    ctx.io.println(f"args: {args}")
}
```

**実装箇所（compiler.fav）:**
- `parse_fn_params` 関数 — `,` 区切りで複数パラメータを受け入れるようループ処理に修正
- `parse_fn_def_after_ret` — 既存の `parse_effects_acc` と連携（変更不要なはず）

### 2. `bench.yml` — `continue-on-error` 除去

**修正前:**
```yaml
- name: Compare with baseline
  continue-on-error: true
  env:
    FAV: ./fav/target/release/fav
  run: |
    $FAV run benchmarks/compare.fav \
      -- --baseline benchmarks/v24.2.0.json \
         --current  benchmarks/latest.json \
         --threshold 5 \
         --emit-md || true
```

**修正後:**
```yaml
- name: Compare with baseline
  env:
    FAV: ./fav/target/release/fav
  run: |
    $FAV run benchmarks/compare.fav \
      -- --baseline benchmarks/v24.2.0.json \
         --current  benchmarks/latest.json \
         --threshold 5 \
         --emit-md || true
```

（"Regression check against baseline" ステップも同様に `continue-on-error` を除去）

### 3. `versions/current.md` 更新

- 「最新安定版」: v70.0.0（記載済みのはず）
- 「進行中バージョン」: v70.1.0（Backlog Blitz）
- 「次に切る版」: v70.1.0（旧情報 v66.9.0 を修正）

---

## Success Criteria

- `compiler.fav` が `fn f(ctx: AppCtx, data: T)` 形式をパースできる
- `fav run benchmarks/compare.fav -- --baseline X --current Y --threshold 5` が正常終了する
- `bench.yml` の Compare / Regression ステップから `continue-on-error: true` が除去されている
- `versions/current.md` に v70.1.0 が「進行中バージョン」として記載されている
- Rust テスト 2 件が pass する:
  - `backlog_compiler_fav_ctx_multiparams`
  - `backlog_bench_yml_compare_strict`

---

## Error Codes

新規エラーコード追加なし。（パーサー内部の修正のみ）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/self/compiler.fav` | `parse_fn_params` に複数パラメータ対応を追加 |
| `.github/workflows/bench.yml` | Compare / Regression ステップから `continue-on-error: true` を除去 |
| `versions/current.md` | 進行中バージョンを v70.1.0 に更新 |
| `fav/Cargo.toml` | version `"70.0.0"` → `"70.1.0"` |
| `CHANGELOG.md` | v70.1.0 エントリを追加 |
| `fav/src/driver.rs` | `backlog_compiler_fav_ctx_multiparams` / `backlog_bench_yml_compare_strict` テストを追加 |
