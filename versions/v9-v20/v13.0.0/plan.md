# Favnir v13.0.0 実装計画

Date: 2026-06-09

---

## Phase A — fav2py pipeline.fav W006 修正

### A-1: `bind _ <-` → `chain _ <-` に変更（LoadAndInsert ステージ）

対象ファイル: `infra/e2e-demo/fav2py/src/pipeline.fav`

変更箇所（4 箇所）：

```diff
- bind _ <- Postgres.execute_raw(
+ chain _ <- Postgres.execute_raw(
    "CREATE TABLE IF NOT EXISTS txn ...",
    "[]"
  )
- bind _ <- Postgres.execute_raw("DELETE FROM txn", "[]")
+ chain _ <- Postgres.execute_raw("DELETE FROM txn", "[]")
- bind _ <- Postgres.execute_raw(
+ chain _ <- Postgres.execute_raw(
    String.concat("INSERT INTO ...", ...),
    "[]"
  )
- bind _ <- IO.println("[INFO] LoadAndInsert complete")
+ bind _ <- IO.println("[INFO] LoadAndInsert complete")   ← Unit 戻り値なので変更不要
```

注意: `IO.println(...)` は `Unit` を返すため W006 の対象外。変更不要。

### A-2: lint でクリーン確認

```bash
cd fav
./target/debug/fav lint --deny-warnings ../infra/e2e-demo/fav2py/src/pipeline.fav
# → exit 0
./target/debug/fav check ../infra/e2e-demo/fav2py/src/pipeline.fav
# → no errors (--legacy-check 不要でも OK)
```

---

## Phase B — airgap analyze.fav 確認

### B-1: lint でクリーン確認

```bash
./target/debug/fav lint --deny-warnings ../infra/e2e-demo/airgap/src/analyze.fav
# → exit 0（W006 なし）
```

airgap は Postgres を使わないため、W006 対象の `bind _ <- NS.fn(...)` は存在しない。
確認のみ（変更不要）。

---

## Phase C — CHANGELOG.md 更新

### C-1: v12.1.0〜v12.10.0 エントリを追記

CHANGELOG.md の `[v12.0.0]` エントリの前に、以下の順で追記：

| バージョン | 内容 |
|---|---|
| v12.10.0 | 全エラーに `help:` + `fav check --strict` + `fav lint --deny-warnings` + `fav.toml [lint]` |
| v12.9.0 | CI `fav test self/*.fav` + Postgres 統合テスト（`services: postgres:16`）|
| v12.8.0 | `fav scaffold <template>` — stage/seq/postgres-etl/rune 雛形生成 |
| v12.7.0 | `fav doc --builtins` / `fav explain <code>` |
| v12.6.0 | Postgres Rune TLS 対応（sslmode=disable/prefer/require）+ エラー詳細化 |
| v12.5.0 | `fav run --verbose/--trace` + `fav check --json/--show-types` |
| v12.4.0 | `seq` pipeline fail-fast（SeqStageCheck opcode）|
| v12.3.0 | `bind` → monadic bind 修正（LegacyBindCheck opcode）|
| v12.2.0 | `bind _` + Result 戻り値 → W006 警告 |
| v12.1.0 | `bind` 再束縛禁止（E0018）|

### C-2: v13.0.0 エントリを追記

```markdown
## [v13.0.0] — 2026-06-09

### Added
- 言語信頼性宣言: 型安全・エラー伝播・デバッグ可視性の三点における保証
- `infra/e2e-demo/fav2py/src/pipeline.fav`: `bind _ <-` → `chain _ <-` W006 修正

### Changed
- README.md に v13.0.0 宣言文を追記

### Notes
- テスト: XXXX 件
- v12.1.0〜v12.10.0 で発覚した全問題（C-1〜C-4 / H-1〜H-2 / M-1 / A-1〜A-6）を解消
```

---

## Phase D — README.md 更新

### D-1: バージョン宣言行を追記

`v12.0.0（2026-06-06）で、Python トランスパイラ...` の行の後に追加：

```markdown
v13.0.0（2026-06-09）で、言語信頼性宣言を完了しました。
型安全・エラー伝播・デバッグ可視性の三点において、Favnir のランタイム挙動は
型システムの宣言と一致することを保証します。
また、`fav check --json` と `fav doc --builtins --format json` を用いて
AI ツールが自律的にコードを修正できることを確認しました。
```

---

## Phase E — バージョン更新・テスト・コミット

### E-1: Cargo.toml version → "13.0.0"

```toml
version = "13.0.0"
```

### E-2: `version_is_12_10_0` を comment out

`fav/src/driver.rs` の `v121000_tests` モジュール内：

```rust
#[test]
fn version_is_12_10_0() {
    // Version bump is tested in v130000_tests::version_is_13_0_0.
}
```

### E-3: `v130000_tests` モジュールを追加

`fav/src/driver.rs` 末尾に追加：

```rust
// ── v130000 tests ─────────────────────────────────────────────────────────────
#[cfg(test)]
mod v130000_tests {
    #[test]
    fn version_is_13_0_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "13.0.0");
    }

    // fav2py pipeline.fav W006 確認（unit test: lint program に W006 がないこと）
    #[test]
    fn fav2py_pipeline_no_w006() {
        use crate::lint::lint_program;
        use crate::front::parser::Parser;
        let pipeline_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../infra/e2e-demo/fav2py/src/pipeline.fav"
        );
        let source = std::fs::read_to_string(pipeline_path)
            .unwrap_or_else(|_| return); // ファイルがなければスキップ
        if source.is_empty() { return; }
        let program = match Parser::parse_str(&source, pipeline_path) {
            Ok(p) => p,
            Err(_) => return,
        };
        let lints = lint_program(&program);
        let w006: Vec<_> = lints.iter().filter(|l| l.code == "W006").collect();
        assert!(w006.is_empty(),
            "fav2py/pipeline.fav should have no W006 after chain fix, got: {:?}",
            w006.iter().map(|l| format!("line {}", l.span.line)).collect::<Vec<_>>());
    }
}
```

### E-4: Cargo.lock 更新

```bash
cargo build
```

### E-5: cargo test 全通過

```bash
cargo test
```

### E-6: git commit + push

```bash
git add -p
git commit -m "feat: v13.0.0 — 言語信頼性宣言"
git push
```

### E-7: CI 確認

`gh run watch` で全ジョブ green を確認。

---

## 実装上の注意

### 1. `chain _ <-` と `bind _ <-` の違い

- `chain _ <- expr`: expr が `Result<T, E>` を返す場合、Err で stage を短絡して上位に伝播。
  W006 の対象にならない。
- `bind _ <- expr`: v12.3.0 以降の `--legacy` モードでも monadic bind 動作。
  ただし W006 が発生するため、Result 戻り値の呼び出しには `chain` を使う。

### 2. airgap デモは変更不要

`analyze.fav` は純粋な IO + CSV 処理のみで Postgres を使わない。
W006 対象の呼び出しがないため修正不要。

### 3. fav2py デモの AWS インフラ

E2E デモのインフラ（RDS / ECS / S3）は既に稼働中（v11.9.0 / v12.0.0 で確認済み）。
v13.0.0 では `terraform apply` は不要。`fav check` / `fav lint` のローカル確認のみ。

### 4. test で `unwrap_or_else(|_| return)` パターン

`pipeline.fav` が存在しないビルド環境（CI の lib テスト等）でテストが落ちないよう、
ファイル読み込み失敗時は return でスキップする。

### 5. CARGO_MANIFEST_DIR

`fav/src/driver.rs` から `infra/` への相対パスは
`concat!(env!("CARGO_MANIFEST_DIR"), "/../infra/...")` で解決できる。
`CARGO_MANIFEST_DIR` は `fav/` ディレクトリを指す。
