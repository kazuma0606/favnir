# Tasks: v85.6.0 — `SapError` 型 + エラーハンドリング（4xx / 5xx / ネットワーク）

Status: COMPLETE

> MILESTONE.md / README.md / `site/content/docs/` の更新は v86.0.0 宣言バージョンで実施する。
> 本バージョンは `types.fav` への型定義追記と Rust テスト追加のみ。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,941 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v85500_tests` が存在することを確認する（v85.5.0 完了済みの証拠）
- [x] `runes/sap-odata/types.fav` に `ODataParams` が存在することを確認する（v85.5.0 追加済み）

## T1: `runes/sap-odata/types.fav` に型定義を追記

- [x] `SapErrorCode` 列挙型を追加する
  - バリアント: `NotFound | Unauthorized | Forbidden | BadRequest | ServerError | NetworkError`
  - HTTP ステータスコードとのマッピングをコメントで記述する
- [x] `SapError` レコード型を追加する
  - `code: SapErrorCode`
  - `message: String`
  - `detail: Option<String>`（OData v4 `innererror` 詳細）

## T2: `mod v85600_tests` を追加

- [x] `mod v85500_tests { ... }` の直後に `#[cfg(test)] mod v85600_tests { ... }` を追加する
- [x] `sap_error_type_exists` テストを実装する
  - `runes/sap-odata/types.fav` に `SapError` が含まれることを確認
  - ファイルパス: `../runes/sap-odata/types.fav`
- [x] `sap_error_code_variants_exist` テストを実装する
  - `runes/sap-odata/types.fav` に `SapErrorCode` が含まれることを確認
  - ファイルパス: `../runes/sap-odata/types.fav`

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,943 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v85.6.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（spec-reviewer / code-reviewer 指摘対応）

- [MED] ロードマップ 2 ファイルの「OData v4 エラーレスポンスのパース」を「型定義（パース処理は v85.9.0）」に修正（spec-reviewer）
- [MED] `sap_error_code_variants_exist` に `NetworkError` バリアントの個別確認を追加（code-reviewer）
- [LOW] `SapError.detail` フィールドに「innererror を単一文字列に簡略化」コメントを補足（code-reviewer）
