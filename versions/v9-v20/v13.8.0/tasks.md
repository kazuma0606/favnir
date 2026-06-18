# v13.8.0 Tasks — ambient effect 禁止（W008 → E0023）

Date: 2026-06-11
Branch: feat/v13-capability-context

---

## Phase A — E0023 エラーカタログ追加

- [x] A-1: `fav/src/error_catalog.rs` — E0023 エントリを追加（E0022 と E0213 の間）
- [x] A-2: `fav/src/driver.rs` `get_help_text` に E0023 ヘルプテキストを追加

---

## Phase B — lint.rs: W008 → E0023 昇格

- [x] B-1: `check_ambient_errors(program)` 関数を追加（E0023 を返す）
- [x] B-2: 内部 `collect_ambient` / `collect_ambient_in_block` / `collect_ambient_in_expr` を `code: &'static str` パラメータ対応に変更
- [x] B-3: E0023 の exemption: `!IO` アノテーション付き関数はスキップ（v13.10.0 で !IO 廃止時に削除）

---

## Phase C — driver.rs: 標準 fav check への統合

- [x] C-1: `cmd_check` に E0023 チェックブロックを追加（非 legacy モード専用）
- [x] C-2: legacy モード（`--legacy`）では E0023 は実行しない（W008 は `--ambient` で確認可）
- [x] C-3: JSON 出力モードでは E0023 チェックをスキップ（フォーマット統一のため後回し）

---

## Phase D — compiler.fav IO 移行

- [x] D-1: `compile_file_after_prog`, `compile_file_after_parse`, `compile_file_after_lex` 削除（デバッグ IO）
- [x] D-2: `compile_bytes(path)` 削除（Rust 側で代替）
- [x] D-3: `compile_bytes_from_src` をメインパブリック API として維持（IO なし）
- [x] D-4: `compile_file_quiet`, `print_bytes`, `main()` を `!IO` アノテーション付きで保持（bootstrap entry point）
- [x] D-5: テスト `"compile_file handles missing file gracefully"` を `compile_bytes_from_src` ベースに更新

---

## Phase E — compiler_fav_runner.rs 更新

- [x] E-1: `compile_file_to_bytes` を Rust でファイル読み込み → `compile_src_str_to_bytes` に委譲するよう変更
  - `compile_bytes(path)` への VM 呼び出しを廃止

---

## Phase F — テスト追加

- [x] F-1: `fav/src/driver.rs` に `v138000_tests` モジュールを追加
- [x] F-2: 以下のテストを実装:
  - [x] `version_is_13_8_0`
  - [x] `e0023_ambient_io_println` — `IO.println` を含む `fn run()` が E0023
  - [x] `e0023_ambient_postgres_raw` — `Postgres.query_raw` が E0023
  - [x] `legacy_mode_allows_ambient` — W008（check_ambient_effects）と E0023（check_ambient_errors）の分岐確認
  - [x] `ctx_based_compiler_fav_compiles` — compiler.fav が E0023 ゼロ
  - [x] `pure_fn_no_e0023` — 純粋関数は E0023 なし
- [x] F-3: `cargo test v138000` 全件パス確認（6/6）

---

## Phase G — バージョンバンプ + コミット

- [x] G-1: `fav/Cargo.toml` → `version = "13.8.0"`
- [x] G-2: `cargo test` 全件パス確認（1484 passed, 0 failed）
- [x] G-3: `git commit -m "feat: v13.8.0 — ambient effect 禁止 (W008 → E0023)"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| E0023 が error_catalog に追加された | ✓ |
| `check_ambient_errors` が E0023 を返す | ✓ |
| 標準 `fav check` が E0023 を出力（非 legacy） | ✓ |
| `--legacy` では E0023 は発生しない | ✓ |
| `!IO` アノテーション付き関数は E0023 が免除される | ✓ |
| compiler.fav のライブラリ関数（`compile_bytes_from_src` 等）が E0023 ゼロ | ✓ |
| `compile_file_to_bytes` が Rust でファイル読み込みに変更 | ✓ |
| `cargo test v138000` 全件パス（6/6） | ✓ |
| `cargo test` 全件パス（1484 passed） | ✓ |
| `CARGO_PKG_VERSION == "13.8.0"` | ✓ |
