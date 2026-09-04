# Tasks: v98.3.0 — SAP Analytics Cloud データプッシュ API（`SacDataset` 型）

Status: COMPLETE

## T0b: ロードマップ修正（spec-reviewer 指摘対応・実装前に完了済み）

- [x] `roadmap-v98.1-v99.0.md` 行 164 の「Rust 側（v98.3.0 で追加）」を「v98.4.0 に延期」と訂正する
- [x] `roadmap-v98.1-v99.0.md` v98.3.0 の「修正ファイル」欄に `sap_odata.fav`（追記）を追加する

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v98.2.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v98200_tests` が存在することを確認する（v98.2.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,239 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `98.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 98.0.0 のまま）

## T1: `runes/sap-odata/sac.fav` を新規作成

- [x] `SacDataset` レコード型を定義する（`model_id: String` / `rows: List<String>`）
- [x] `sac_push_mock(dataset: SacDataset) -> String` ヘルパー関数を実装する
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）

## T2: `runes/sap-odata/sap_odata.fav` に追記

- [x] `use sap_odata.sac` を use セクションに追加する
- [x] `public type SacDataset = sac.SacDataset` を re-export する
- [x] `public fn sac_push_mock(...)` を re-export する

## T3: `fav/src/driver.rs` に `mod v98300_tests` を追加

- [x] `mod v98200_tests` の直後に `mod v98300_tests`（2 テスト）を追加する:
  - `sac_fav_exists`: `../runes/sap-odata/sac.fav` の存在を確認
  - `sac_fav_has_sac_dataset`: `SacDataset` が含まれることを確認
- [x] `mod v98300_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,241 tests, 0 failures であることを確認する

## T5: `CHANGELOG.md` に v98.3.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v98.3.0]` エントリを追加する

## T6: `versions/current.md` 更新

- [x] `最終更新:` ヘッダーを `v98.3.0` に更新する
- [x] 最新安定版を `v98.3.0` に更新する（テスト数 4,241）

<!-- Effect::SapAnalytics の Rust 実装は v98.4.0 で対応予定（本バージョンはスコープ外） -->
<!-- site MDX ドキュメントは v98.8.0 で対応予定（本バージョンはスコープ外） -->
<!-- MILESTONE.md 更新は宣言版（v99.0.0）で対応予定（patch version は対象外） -->

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後・T5/T6 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
