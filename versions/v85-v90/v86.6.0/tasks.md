# Tasks: v86.6.0 — シナリオ 1: マスタデータ同期（BusinessPartner → S3）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,963 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v86500_tests` が存在することを確認する（v86.5.0 完了済みの証拠）
- [x] `infra/e2e-demo/sap-odata/` ディレクトリが存在することを確認する（v85.9.0 で作成済み）

## T1: `CHANGELOG.md` に v86.6.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に v86.6.0 エントリを追加する（テストモジュール追加より先に実施）

## T2: `infra/e2e-demo/sap-odata/pipeline.fav` を新規作成

- [x] `sync_business_partners()` 関数を実装する（`import rune "s3"` + `sap_odata` 関数使用）
- [x] v86.8.0 以降の Registry 切り替え予定をコメントで明記する

## T3: `mod v86600_tests` を追加

- [x] `mod v86500_tests { ... }` の直後に `#[cfg(test)] mod v86600_tests { ... }` を追加する
- [x] `sap_e2e_pipeline_fav_exists` テストを実装する
- [x] `sap_e2e_pipeline_contains_sync_business_partners` テストを実装する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,965 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [MED] `pipeline.fav` に `sap_odata` 参照方針の説明コメントを追加（v86.8.0 で `import rune "sap-odata"` に切り替える旨、現時点でローカル直接参照の理由を明示）
- [LOW] `BusinessPartnerFilter` リテラルのフィールド順序を型定義順（`country / category / changed_after / top`）に修正
- [LOW] エフェクト注釈欠落: スタブ関数全体の問題（sap_odata.fav 含む）のため v86.6.0 では対応なし
