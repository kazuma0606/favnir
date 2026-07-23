# Spec: v49.7.0 — セキュリティ審査 2.0

Date: 2026-07-18
Status: Draft

---

## 概要

import 2.0 のパストラバーサル攻撃を拒否し、`fav install` のパッケージ名バリデーションを実施する。
`driver.rs` に `validate_import_path` / `validate_rune_name` ヘルパーを追加してセキュリティを強化する。

---

## 背景

v49.1〜v49.6 で統合・安定化を進めてきたが、セキュリティ観点の審査が残っていた。
import パスに `../../etc/passwd` などを渡されると意図しないファイルアクセスが起きる可能性がある。
パッケージ名に任意文字列を許すと、シェルインジェクションやレジストリ汚染のリスクがある。

---

## 仕様

### `validate_import_path(path: &str) -> Result<(), String>`

- `..` コンポーネントを含むパスを拒否（パストラバーサル防止）
- 空文字列を拒否
- `Ok(())` を返す = 安全なパス
- `Err(msg)` を返す = 拒否理由を含むエラーメッセージ

拒否条件:
1. パスが空
2. パスを `/` で分割したとき、いずれかのコンポーネントが `..` に等しい（パストラバーサル防止）
3. `\\` を含む（Windows パス区切り混入防止）

### `validate_rune_name(name: &str) -> Result<(), String>`

- 英数字（`a-z`, `A-Z`, `0-9`）と `-` のみ許可
- 空文字列を拒否
- 先頭・末尾が `-` のものを拒否
- 連続 `--` を拒否

---

## テスト

`v497000_tests` モジュールに 2 件追加（`v496000_tests` の直前）:

1. `import_path_traversal_rejected`
   - `validate_import_path("../../etc/passwd")` が `Err` を返すことを確認
   - `validate_import_path("./stages/validate")` が `Ok` を返すことを確認

2. `install_invalid_name_rejected`
   - `validate_rune_name("my-rune")` が `Ok` を返すことを確認
   - `validate_rune_name("../evil")` が `Err` を返すことを確認（`/` を含む — 許可外文字）
   - `validate_rune_name("my rune")` が `Err` を返すことを確認（スペース含む）

---

## 完了条件

- `cargo test` 3083 tests passed, 0 failed（3081 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v49.7.0 エントリ追加
- `versions/current.md` を v49.7.0 に更新
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.7.0 実績を記入
