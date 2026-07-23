# Spec: v50.0.0 — Production 2.0 宣言 ★クリーンアップ

Date: 2026-07-18
Status: Draft

---

## 概要

Favnir v50.0 — Production 2.0 を正式宣言する。
v49.1〜v49.9 の全機能統合・安定化・セキュリティ審査を経て、
`return` ガード節・成熟した stdlib・明確なモジュールシステム・インラインテストが揃った
「迷わず使える実用言語」として Favnir を宣言する。

`cargo clean`（★クリーンアップ）を実施し、ビルド成果物を削除する。

---

## 宣言文

> 「`return` による安全なガード節、成熟した標準ライブラリ、
>  明確なモジュールシステム、インラインテストが揃い、
>  Favnir は迷わず使える実用言語になった。
>
>  これが Favnir v50.0 — Production 2.0 の姿である。」

---

## 仕様

### `README.md` 更新

`README.md` に `"Language Maturity"` および `"v50"` への言及を追加する。
`readme_mentions_language_maturity` テストが `README.md` に `"Language Maturity"` を含むことを確認する。

### `MILESTONE.md` 確認

v49.8.0 で追加済み。`"Language Maturity"` が含まれることを確認する（追加作業不要）。

### `v50000_tests` モジュール（4 テスト）

`v499000_tests` の直前に追加:

1. `cargo_toml_version_is_50_0_0`
   - `include_str!("../Cargo.toml")` で読み込み
   - `version = "50.0.0"` が含まれることを確認

2. `changelog_has_v50_0_0`
   - `include_str!("../../CHANGELOG.md")` で読み込み
   - `"v50.0.0"` が含まれることを確認

3. `milestone_has_language_maturity`
   - `include_str!("../../MILESTONE.md")` で読み込み
   - `"Language Maturity"` が含まれることを確認
   - `"v50.0.0"` が含まれることを確認（v498000_tests と同名だが内容を同一に揃えて差別化を明確にする）

4. `readme_mentions_language_maturity`
   - `include_str!("../../README.md")` で読み込み
   - `"Language Maturity"` が含まれることを確認

### ★クリーンアップ

`cargo clean` を実施してビルド成果物を削除する。
その後 `cargo test` を再実行して全通過を確認する。
`fav/tmp/hello.fav` が削除されていた場合は復元する（内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`）。

---

## 完了条件

> 注: Step 2 でテストを追加した直後は `readme_mentions_language_maturity` / `changelog_has_v50_0_0` が fail する。
> Step 1（README 更新）と Step 3（CHANGELOG + Cargo.toml 更新）完了後に初めて全 pass する。
> plan の Step 順序通りに実施すること。

- `cargo test` 3091 tests passed, 0 failed（3087 + 4 件）
- `v50000_tests` 4 件すべて pass
- `cargo clean` 実施済み
- `cargo test` clean 後も全通過
- `MILESTONE.md` に `"Language Maturity"` が含まれる
- `README.md` に `"Language Maturity"` が含まれる
- `CHANGELOG.md` に v50.0.0 エントリが含まれる
- `cargo clippy -- -D warnings` クリーン
- `versions/current.md` を v50.0.0 に更新
