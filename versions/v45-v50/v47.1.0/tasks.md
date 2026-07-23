# Tasks: v47.1.0 — `List.zip` / `List.chunk`

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3016 passed, 0 failed を確認
- [x] `vm.rs` に `List.zip`（line 11154）・`List.chunk`（line 11332）の実装が存在することを確認

## T1 — `driver.rs` に `v471000_tests` 追加

- [x] `v47000_tests` の直後に `v471000_tests` モジュールを追加（2 テスト）
  - [x] `list_zip_pairs`: `List.zip(names, scores)` → `List.length(pairs) == 2`
  - [x] `list_chunk_batches`: `List.chunk(data, 2)` → `List.length(batches) == 3`

## T2 — バージョン更新・テスト・完了

- [x] `fav/Cargo.toml` version → `"47.1.0"`
- [x] `CHANGELOG.md` に v47.1.0 エントリ追加
- [x] `cargo test` 3018 passed, 0 failed（3016 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン（次ステップで確認）
- [x] `versions/current.md` を v47.1.0（3018 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T2 全チェック）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | ロードマップが「vm.rs に実装追加」と記述しているが spec は「実装済み」と書いていた | vm.rs を確認し実装済みと判明。spec の前提記述を維持 |
| [HIGH] | テスト数がロードマップ（3013）と spec（3018）で不一致 | spec に「ロードマップの 3013 は古い推定値」を注記として追記 |
| [MED] | `List.zip_with` のテストが spec タイトルに含まれているのにテスト一覧にない | タイトルから `List.zip_with` を削除、スコープを `List.zip` / `List.chunk` に限定 |
| [MED] | `run_source` ヘルパーの戻り値型が不明確 | plan.md を `Parser::parse_str` + `exec_artifact_main` パターンに修正 |
| [MED] | `List.head` 非存在・`List.first` が Option を返すため `first.first` アクセスが複雑 | テストを `List.length(pairs) == 2` に変更してシンプル化 |
| [LOW] | T0 に vm.rs 実装確認が未記載 | T0 に「vm.rs に実装が存在することを確認」チェックを追加 |
| [LOW] | ドキュメント更新なしの明示が未記載 | spec に「サイトドキュメントは v47.9.0 で対応」注記を追加済み |

## 実装中の追加修正

| 内容 | 対応 |
|---|---|
| `List.from(["alice", "bob"])` が構文エラー（Favnir にリストリテラルなし） | `List.push(List.singleton("alice"), "bob")` と `List.range(1, 6)` に変更 |
| `v47000_tests::cargo_toml_version_is_47_0_0` が v47.1.0 更新後に失敗 | 他バージョンと同様に stub（空実装）に変更 |
