# v64.6.0 タスクリスト

Status: COMPLETE
Version: 64.6.0
Base tests: 3441
Target tests: 3443

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3441 tests passed, 0 failed を確認
- [x] `driver.rs` に `v64600_tests` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `v64500_tests` が存在することを確認（`v64600_tests` の挿入位置）
- [x] `toml.rs` の `LintTomlConfig` に `perf` フィールドがないことを確認（新規追加）
- [x] `toml.rs` の `[lint]` セクションが `strict` キーをパースすることを確認（`perf` も同様にパース）
- [x] `lint.rs` に `LintConfig { perf: bool }` が存在することを確認（v63.6.0 実装済み）
- [x] `lint.rs` に W041 が `config.perf || config.strict` でゲートされていることを確認
- [x] `driver.rs` の `cmd_lint` が `perf: false` をハードコードしていることを確認（更新対象）
- [x] `parse_fav_toml_pub` が `toml.rs` に存在することを確認（v64.2.0 実装済み）
- [x] `driver.rs` / `resolver.rs` / `checker.rs` に `LintTomlConfig { ... }` のリテラルが存在しないことを確認（`toml.rs` の 1 箇所のみ）

**スコープ注記**: `main.rs` への `--perf` CLI フラグ追加と `cmd_lint` シグネチャ変更は後送り（v64.7 以降）

---

## T1: `toml.rs` — `LintTomlConfig` 更新

- [x] `LintTomlConfig` 構造体に `pub perf: Option<bool>` フィールドを追加
  - [x] コメント `/// v64.6.0: perf = true で W041 等のパフォーマンス lint を有効化。` を付与
- [x] `parse_fav_toml` の `[lint]` セクション struct literal（`unwrap_or(LintTomlConfig {...})`）に `perf: None` を追加
- [x] `[lint]` セクションの `parse_kv` 処理に `perf` キーパースを追加
  - [x] `else if key == "perf" { current.perf = Some(val.trim() == "true"); }` を `strict` の直後に追加

---

## T2: `driver.rs` — `cmd_lint` の `perf` toml 連携

- [x] `cmd_lint` 内の `let run_config = LintConfig { strict: strict_mode, perf: false };` を更新
  - [x] `let perf_mode = lint_config.as_ref().and_then(|c| c.perf).unwrap_or(false);` を追加
  - [x] `LintConfig { strict: strict_mode, perf: perf_mode }` に変更

---

## T3: `driver.rs` — `v64600_tests` 追加

- [x] `// -- v64500_tests` コメント行の直前に `v64600_tests` を挿入
  - [x] `lint_perf_flag_enables_w041`（`perf: true` で W041 が発火することを確認）
  - [x] `lint_toml_perf_setting`（`[lint] perf = true` が `lint.perf == Some(true)` にパースされることを確認）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v64600_tests` で 2 件 PASS
  - [x] `lint_perf_flag_enables_w041` PASS
  - [x] `lint_toml_perf_setting` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3443 tests passed, 0 failed を確認

---

## T5: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v64.6.0 エントリを追加
- [x] `versions/roadmap/roadmap-v64.1-v65.0.md` v64.6.0 セクションに実績追記（3443 tests、`--perf` CLI 後送りを明記）
- [x] `versions/current.md` の「進行中」を v64.6.0（3443 tests）に更新
- [x] `MILESTONE.md` は v65.0 で更新（本バージョンでは不要）
- [x] tasks.md を COMPLETE に更新（本ファイル）
