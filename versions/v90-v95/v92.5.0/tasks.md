# Tasks: v92.5.0 — W060 N+1 lint ルール追加

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,105 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v92400_tests` が存在することを確認する（v92.4.0 完了済みの証拠）
- [x] `runes/sap-odata/query_builder.fav` に `public type Page` が含まれることを確認する（v92.4.0 完了済みの証拠）
- [x] `runes/sap-odata/query_builder.fav` に `public fn fetch_all_pages` が含まれることを確認する（v92.4.0 完了済みの証拠）
- [x] `fav/tmp/hello.fav` が存在することを確認する
- [x] `fav/src/lint.rs` を Read し、既存 lint ルールの構造（check 関数・メッセージ形式）を確認する（W059 が最高値、W060 が次の空き番号）

## T1: `lint.rs` に W060 ルールを追加する

- [x] 既存 lint ルールパターン（W001〜W059）に従って W060 を追加する（W059 の直後）
- [x] 警告メッセージに `"W060"` と `"N+1"` を含める
- [x] `List.map` / `List.flat_map` コールバック内の `ctx.sap.*` 呼び出しを検出するロジックを実装する

## T2: `driver.rs` に `mod v92500_tests` を追加する

- [x] `mod v92400_tests { ... }` の直後に `#[cfg(test)] mod v92500_tests { ... }` を追加する
- [x] `w060_lint_rule_defined` テストを実装する（`src/lint.rs` に `W060` 含有確認）
- [x] `w060_lint_message_mentions_n_plus_1` テストを実装する（`src/lint.rs` に `N+1` 含有確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,107 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T4: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
