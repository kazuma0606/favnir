# v70.1.0 実装計画 — Backlog Blitz（積み残し一掃）

Date: 2026-08-08

---

## 事前確認（T0）

- [ ] `fav/Cargo.toml` のバージョンが `70.0.0` であることを確認
- [ ] `cargo test` が全 pass（3559 tests）であることを確認
- [ ] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）
- [ ] `.github/workflows/bench.yml` の現状（`continue-on-error: true` があること）を確認

---

## ステップ 1: compiler.fav — `parse_fn_params` 複数パラメータ対応

**ファイル**: `fav/self/compiler.fav`

`parse_fn_params` 関数を修正して、`,` 区切りの複数パラメータを正しく解析できるようにする。

```
現状: parse_fn_params が最初のパラメータ後に ',' を認識できずパースエラー
修正: ループまたは再帰で ',' があれば次のパラメータを読み続ける
```

実装のポイント:
1. `parse_fn_params` 内で `,` トークンを消費後に次のパラメータを読むループを追加
2. `)` に到達したらパラメータリストを確定する
3. 既存の単一パラメータケースとの後方互換性を維持する

---

## ステップ 2: Rust テスト 2 件を driver.rs に追加

**ファイル**: `fav/src/driver.rs`

```rust
#[cfg(test)]
mod v701000_tests {
    fn compile_src(src: &str) -> Result<Vec<u8>, String> {
        use crate::frontend::{lexer::Lexer, parser::Parser};
        use crate::middle::compiler::compile;
        let tokens = Lexer::new(src).tokenize().map_err(|e| e.to_string())?;
        let ast = Parser::new(tokens).parse().map_err(|e| e.to_string())?;
        compile(&ast).map_err(|e| e.to_string())
    }

    #[test]
    fn backlog_compiler_fav_ctx_multiparams() {
        // compiler.fav が複数パラメータ関数をパースできることを確認
        let src = r#"
fn helper(ctx: AppCtx, data: String) -> Result<Unit, String> {
    ctx.io.println(data)
}
fn main(ctx: AppCtx) -> Result<Unit, String> {
    helper(ctx, "hello")
}
"#;
        let result = compile_src(src);
        assert!(result.is_ok(), "multi-param fn should parse: {:?}", result);
    }

    #[test]
    fn backlog_bench_yml_compare_strict() {
        // bench.yml の Compare ステップが continue-on-error なしになっていることを確認
        let bench_yml = include_str!("../../.github/workflows/bench.yml");
        assert!(
            !bench_yml.contains("continue-on-error: true"),
            "bench.yml should not have continue-on-error: true"
        );
    }
}
```

---

## ステップ 3: bench.yml から `continue-on-error: true` を除去

**ファイル**: `.github/workflows/bench.yml`

以下のステップから `continue-on-error: true` を除去する:
- "Compare with baseline" ステップ
- "Regression check against baseline" ステップ

（"Run benchmarks" ステップの `continue-on-error: true` は benchmarks/suite/run_all.sh に
依存するため、そのまま維持する）

---

## ステップ 4: `versions/current.md` 更新

**ファイル**: `versions/current.md`

- 「進行中バージョン」を v70.1.0（Backlog Blitz）に更新
- 「次に切る版」を v70.1.0 に更新（旧情報を修正）

---

## ステップ 5: `fav/Cargo.toml` バージョン更新

`"70.0.0"` → `"70.1.0"`

---

## ステップ 6: `CHANGELOG.md` にエントリ追加

最上部に v70.1.0 のエントリを追加する:

```markdown
## [v70.1.0] — 2026-08-08 — Backlog Blitz（積み残し一掃）

### Fixed
- `compiler.fav` の `parse_fn_params` を修正 — 複数パラメータ関数 `fn f(ctx: AppCtx, data: T)` を正しくパースできるよう対応
- `.github/workflows/bench.yml` の Compare / Regression ステップから `continue-on-error: true` を除去 — CI が完全グリーンになった
- `versions/current.md` を v70.0.0 完了・v70.1.0 進行中に更新

### Added
- `v701000_tests`: 2 件追加（3559 → 3561 tests）
  - `backlog_compiler_fav_ctx_multiparams`
  - `backlog_bench_yml_compare_strict`
```

---

## ステップ 7: 動作確認

```bash
# テスト全 pass 確認（3561 tests）
cd fav && cargo test 2>&1 | tail -5

# compiler.fav が複数パラメータ関数をパースできるか確認（コンパイルエラーが出ないことを確認する目的）
echo 'fn helper(ctx: AppCtx, data: String) -> Result<Unit, String> { ctx.io.println(data) }
fn main(ctx: AppCtx) -> Result<Unit, String> { helper(ctx, "hello") }' > /tmp/multi_param_test.fav
./target/debug/fav check /tmp/multi_param_test.fav
```

---

## 実装順序まとめ

```
T0: 事前確認（バージョン・テスト数・ファイル存在確認）
1: compiler.fav parse_fn_params 修正
2: driver.rs テスト 2 件追加
3: bench.yml continue-on-error 除去
4: versions/current.md 更新
5: Cargo.toml バージョン更新
6: CHANGELOG.md エントリ追加
7: cargo test 全 pass 確認（3561 tests）
```
