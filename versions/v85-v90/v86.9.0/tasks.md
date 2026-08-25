# Tasks: v86.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,969 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v86800_tests` が存在することを確認する（v86.8.0 完了済みの証拠）
- [x] `runes/sap-odata/rune.toml` の version が `86.8.0` であることを確認する
- [x] `rune-registry/src/main.fav` に `!Effect` 注釈が残っていないことを確認する（v86.8.0 で修正済み）

## T1: `CHANGELOG.md` に v86.9.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に v86.9.0 エントリを追加する（テストモジュール追加より先に実施）

## T2: `mod v86900_tests` を追加

- [x] `mod v86800_tests { ... }` の直後に `#[cfg(test)] mod v86900_tests { ... }` を追加する
- [x] `sap_master_data_business_partner_crud_covered` テストを実装する（4 関数の存在確認）
- [x] `sap_master_data_scenario1_pipeline_exists` テストを実装する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,971 tests, 0 failures であることを確認する

## T4: `import rune "sap-odata"` 動作確認（手動）

- [x] `curl -s -H "x-fav-token: fav-registry-v1-dk9p2mxw4qhz" https://32qp3qwhdh.execute-api.ap-northeast-1.amazonaws.com/runes/sap-odata` を実行する
- [x] レスポンスに `"name":"sap-odata"` が含まれることを確認する（HTTP 200）
  - 結果: `{"name":"sap-odata","version":"86.8.0","description":"SAP S/4HANA OData v4 クライアント ..."}` → OK
  - 補足: Registry 未登録だったため Python で zip を作成し POST 登録してから確認した

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
- 注: `versions/current.md` の更新は v87.0.0 宣言時に実施する

## 修正事項（code-reviewer 指摘対応）

- [MED] `rune-registry/src/main.fav` の `public fn main` に副作用コメントを追加（`!Env` / `!AWS` / `!Io`）— `!Effect` 構文は E0374 廃止のためコメントで代替
- [LOW] `fav/src/driver.rs` 末尾改行を追加（`}` → `}\n`）
- [LOW] `sap_master_data_scenario1_pipeline_exists` テストに「Existence check」コメントを追加（関数名と内容の乖離を明記）
