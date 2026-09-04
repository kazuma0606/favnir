# Tasks: v99.1.0 — OAuth2 PKCE / SAP BTP Trust Configuration

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v99.0.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v99.0.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v99000_tests` が存在することを確認する（v99.0.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,257 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `99.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 99.0.0 のまま）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: btp_auth.fav を新規作成

- [x] `runes/sap-odata/btp_auth.fav` を新規作成する
- [x] ファイル先頭コメントに `-- runes/sap-odata/btp_auth.fav` が含まれることを確認する
- [x] `BtpCredential` 型（`client_id` / `client_secret` / `token_url` / `scope`）が定義されていることを確認する
- [x] `BtpToken` 型（`access_token` / `expires_in` / `token_type`）が定義されていることを確認する
- [x] `acquire_token_mock(cred: BtpCredential) -> BtpToken` 関数が実装されていることを確認する
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）

## T1.5: sap_odata.fav に use と re-export を追加

- [x] `runes/sap-odata/sap_odata.fav` の `use` 宣言ブロックに `use sap_odata.btp_auth` を追加する
- [x] `sap_odata.fav` 末尾に `-- BTP 認証型 re-export（v99.1.0〜）` セクションを追加する
- [x] `BtpCredential` / `BtpToken` / `acquire_token_mock` の 3 シンボルが re-export されていることを確認する

## T2: driver.rs に mod v99100_tests を追加

- [x] `mod v99000_tests` の直後に `mod v99100_tests`（2 テスト）を追加する:
  - `btp_auth_fav_exists`: `runes/sap-odata/btp_auth.fav` の存在を確認
  - `btp_auth_has_btp_credential`: `BtpCredential` が含まれることを確認
- [x] `mod v99100_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T3: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,259 tests, 0 failures であることを確認する

## T4: CHANGELOG.md に v99.1.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v99.1.0]` エントリを追加する

## T5: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v99.1.0` に更新する
- [x] 最新安定版を `v99.1.0` に更新する（テスト数 4,259）

## T6: ロードマップのテスト数を実績値ベースに修正

- [x] `versions/roadmap/roadmap-v99.1-v100.0.md` のバージョン一覧表テスト数を実績値（4,257 起点）に修正する:
  - v99.1.0: `4257 + 2 = 4259`
  - v99.2.0: `4259 + 2 = 4261`
  - v99.3.0: `4261 + 2 = 4263`
  - v99.4.0: `4263 + 2 = 4265`
  - v99.5.0: `4265 + 2 = 4267`
  - v99.6.0: `4267 + 2 = 4269`
  - v99.7.0: `4269 + 2 = 4271`
  - v99.8.0: `4271 + 2 = 4273`
  - v99.9.0: `4273 + 2 = 4275`
  - v100.0.0: `4275 + 4 = 4279`

<!-- MILESTONE.md 更新は宣言版（v100.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v99.8.0 で対応予定（本バージョンはスコープ外） -->

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後・T4/T5 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 指摘対応（実装後）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [MED] | `BtpToken` / `acquire_token_mock` の存在確認が欠落 | `btp_auth_has_btp_credential` テストに `BtpToken` / `acquire_token_mock` のアサーションを追加（テスト数 4,259 維持） |
| [LOW] | `btp_auth_has_btp_credential` の `expect` にバージョン注記欠落 | `(v99.1.0)` を追加 |
| [LOW] | `sap_odata.fav` の `acquire_token_mock` 戻り型が `btp_auth.BtpToken` のまま | re-export 済みエイリアス `BtpToken` に統一 |
