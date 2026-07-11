# v39.6.0 実装計画 — `fav audit`

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/fav_audit.rs` | 新規作成 | `cmd_audit(check_mode)` / `collect_rune_deps()` スタブ実装 |
| `fav/src/main.rs` | 変更 | `mod fav_audit;` 追加 + `Some("audit")` ディスパッチアーム追加 |
| `fav/src/driver.rs` | 変更 | `v39500_tests::cargo_toml_version_is_39_5_0` スタブ化 / `v39600_tests` 追加（2 テスト）|
| `fav/Cargo.toml` | 更新 | `version = "39.5.0"` → `"39.6.0"` |
| `CHANGELOG.md` | 追記 | `[v39.6.0]` エントリ追加 |
| `versions/roadmap/roadmap-v39.1-v40.0.md` | 更新 | v39.6.0 を完了済みにマーク（✅）|
| `versions/current.md` | 更新 | 最新安定版 v39.6.0、次に切る版 v39.7.0 |
| `versions/v36-v40/v39.6.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T7 全チェック）|

## 注記: Step 番号と tasks.md T 番号の対応

| plan.md Step | tasks.md T 番号 | 内容 |
|---|---|---|
| Step 1 | T1 | CHANGELOG 追加 |
| Step 2 | T2 | fav_audit.rs 新規作成 |
| Step 3 | T3 | main.rs — mod + dispatch 追加 |
| Step 4 | T4 | driver.rs スタブ化 |
| Step 5 | T5 | driver.rs v39600_tests 追加 |
| Step 6 | T6 | Cargo.toml バージョン更新 |
| Step 7 | T7 | cargo test 実行 + ドキュメント更新 |

tasks.md には T0（事前確認）が先頭に挿入されているため、Step N は T(N) に対応する（T0 を除く）。

## 実装順序

### Step 1: CHANGELOG.md に [v39.6.0] エントリ追加（tasks.md: T1）

`## [v39.5.0]` ヘッダ行の直前に挿入:

```markdown
## [v39.6.0] — YYYY-MM-DD

### Added
- `fav/src/fav_audit.rs` — `fav audit`（依存 Rune ライセンス一覧）/ `fav audit --check`（GPL・CVE 検出、exit 1）追加
- `v39600_tests` 2 テスト追加（meta 2 件）

---
```

**注意**: セパレータは `—`（全角ダッシュ U+2014）。日付は実装当日の `YYYY-MM-DD` 形式。

### Step 2: `fav/src/fav_audit.rs` 新規作成（tasks.md: T2）

spec.md §1 の内容で作成:
- `pub fn cmd_audit(check_mode: bool) -> Result<(), String>`
- `fn collect_rune_deps() -> Result<Vec<String>, String>` — スタブ（空リスト返却 + TODO コメント）
- `check_mode=true` 時: GPL 含む Rune を violation として `eprintln!` + `process::exit(1)`
- `check_mode=false` 時: 全 Rune を `println!` + 件数表示

### Step 3: `fav/src/main.rs` — `mod fav_audit;` + `Some("audit")` アーム追加（tasks.md: T3）

1. Read で `mod policy;` の行番号を確認
2. `mod policy;` の直後に `mod fav_audit;` を追加
3. Read で `Some("policy")` ディスパッチアームの行番号を確認
4. `Some("policy")` アームの直後に `Some("audit")` アームを追加（spec.md §2 参照）

**注意**: Step 2（fav_audit.rs 作成）完了後に Step 3 を実施すること。

### Step 4: `driver.rs` — `v39500_tests::cargo_toml_version_is_39_5_0` スタブ化（tasks.md: T4）

Grep で `cargo_toml_version_is_39_5_0` の行番号を確認 → ライブアサーションを:
```rust
// Stubbed: version bumped to 39.6.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v39_5_0` / `tenant_rune_db_schema` / `tenant_rune_s3_prefix` はスタブ化しない。

### Step 5: `driver.rs` — `v39600_tests` モジュール追加（tasks.md: T5、Step 1 完了後）

`v39500_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §3 のコードブロックに従う:
- imports 不要（`include_str!` のみ使用）
- 2 テスト: `cargo_toml_version_is_39_6_0`（NOTE コメント付き） / `changelog_has_v39_6_0`

### Step 6: Cargo.toml バージョン更新（tasks.md: T6）

Step 1〜5 完了後に `39.5.0` → `39.6.0` に更新。

### Step 7: `cargo test` 実行 + ドキュメント更新（tasks.md: T7）

```
cargo test 2>&1 | grep "test result"
```

期待: ≥ 2803 passed, 0 failed

その後:
- `versions/roadmap/roadmap-v39.1-v40.0.md` の v39.6.0 を ✅ にマーク
- `versions/current.md` を v39.6.0（最新安定版）・v39.7.0（次に切る版）に更新
- `versions/v36-v40/v39.6.0/tasks.md` を COMPLETE ステータスに更新

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 5 (driver tests, changelog_has_v39_6_0)
Step 2 (fav_audit.rs) ───────────────────────────► Step 3 (main.rs mod + dispatch)
Step 3 (main.rs) ────────────────────────────────► Step 7 (cargo test — コンパイル検証)
Step 4 (stub v39500) ────────────────────────────► Step 5 (driver tests)
Step 5 (v39600_tests) ───────────────────────────► Step 6 (Cargo.toml bump)
Step 6 (Cargo.toml) ─────────────────────────────► Step 7 (cargo test)
Step 7 (all pass) ───────────────────────────────► docs 更新
```

## リスク

| リスク | 対処 |
|---|---|
| `mod fav_audit;` を先に追加すると `fav_audit.rs` がなくてコンパイルエラー | Step 2 → Step 3 の順序を厳守 |
| `fav audit` と既存 `runes/audit/` の名前衝突 | Rust モジュール名を `fav_audit` とすることで回避済み |
| `Some("policy")` アームの挿入位置がずれる | Read で前後を確認してから Edit |
| `gen` 予約語（Rust 2024）| v39.6.0 テストでは `cargo`/`src`/`runes` 変数のみ使用 — 問題なし |
