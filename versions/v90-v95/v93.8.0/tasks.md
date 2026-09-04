# Tasks: v93.8.0 — サイトドキュメント更新

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,134 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v93700_tests` が存在することを確認する（v93.7.0 完了済みの証拠）
- [x] `site/content/docs/runes/sap-odata.mdx` が存在することを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `site/content/docs/cli/infer.mdx` を新規作成する

- [x] `site/content/docs/cli/run.mdx` の構造を参考に MDX ファイルを新規作成する
- [x] `--from sap --metadata <url>` コマンド例を記載する
- [x] `--from sap --metadata-file <path>` コマンド例（CI/オフライン向け）を記載する
- [x] ファイルに `sap-metadata` という文字列を含める（テスト `docs_infer_mentions_sap_metadata` の対象）

## T2: `site/content/docs/runes/sap-odata.mdx` を更新する

- [x] 既存ファイルを読む（注意: 現時点で `metadata` は含まれていないため、マッピング表の追加が必須）
- [x] EDM 型 → Favnir 型マッピング表を追加する（`Edm.String`→`String`、`Edm.Int32`/`Edm.Int64`→`Int` 等）— 見出しに `metadata` を含めること
- [x] `NavigationProperty` → `ExpandClause` ヘルパー対応表を追加する
- [x] 追加後に `metadata` という文字列がファイルに含まれていることを確認する（テスト `docs_sap_odata_mentions_metadata_infer` の対象）

## T3: `driver.rs` に `mod v93800_tests` を追加する

- [x] `mod v93700_tests { ... }` の直後に `#[cfg(test)] mod v93800_tests { ... }` を追加する
- [x] `docs_infer_mentions_sap_metadata` テストを実装する（`../site/content/docs/cli/infer.mdx` に `sap-metadata` が含まれる）
- [x] `docs_sap_odata_mentions_metadata_infer` テストを実装する（`../site/content/docs/runes/sap-odata.mdx` に `metadata` が含まれる）
- [x] パスは `../site/` 形式を使用する（`fav/` ディレクトリ起点のテスト実行に対応）

## T4: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,136 tests, 0 failures であることを確認する

## T6: CHANGELOG.md を更新する

- [x] `CHANGELOG.md` に v93.8.0 のエントリを追加する

## T6b: ロードマップ本文を確認する

- [x] `versions/roadmap/roadmap-v93.1-v94.0.md` の v93.8.0 本文が `4134 + 2 = 4136` になっていることを確認する（v93.7.0 T6b で修正済み）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T7: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
