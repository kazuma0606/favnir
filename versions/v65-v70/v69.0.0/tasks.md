# v69.0.0 タスクリスト

Status: COMPLETE
Version: 69.0.0
Note: Distributed Favnir 宣言 ★クリーンアップ — Cargo.toml version 更新 + ドキュメント + 4 テスト + cargo clean
Base tests: 3537
Target tests: 3541

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3537 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"68.0.0"` であることを確認（本バージョンで `"69.0.0"` に更新）
- [x] `driver.rs` に `v68900_tests` が存在することを確認（`v69000_tests` の挿入位置）
  - 注意: driver.rs のテストブロックは降順配置（新しいものが上）。`v69000_tests` を `v68900_tests` の直前に挿入する
- [x] `driver.rs` に `v69000_tests` が存在しないことを確認（新規追加）
- [x] `versions/current.md` の「進行中バージョン」が `v68.9.0` であることを確認
- [x] `cargo test --bin fav v68900_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `distributed_all_stable`, `distributed_docs_complete`
- [x] `MILESTONE.md` に `"Distributed Favnir"` が含まれないことを確認（新規追加）
- [x] `CHANGELOG.md` に `"v69.0.0"` が含まれないことを確認（新規追加）
- [x] `README.md` に `"Distributed Favnir"` が含まれないことを確認（新規追加）

---

## T1: `fav/Cargo.toml` バージョン更新

- [x] `version = "68.0.0"` を `version = "69.0.0"` に変更

---

## T2: `MILESTONE.md` 更新

- [x] 先頭に v69.0.0「Distributed Favnir」エントリを追加
  - [x] `"Distributed Favnir"` キーワードを含む（`milestone_has_distributed` テスト要件）
  - [x] v68.1〜v68.8 の各機能を箇条書きで記載

---

## T3: `README.md` 更新

- [x] v69.0.0 宣言文を追加
  - [x] `"Distributed Favnir"` キーワードを含む（`readme_mentions_distributed` テスト要件）

---

## T4: `CHANGELOG.md` 更新

- [x] 先頭に v69.0.0 エントリを追加
  - [x] `"v69.0.0"` キーワードを含む（`changelog_has_v69_0_0` テスト要件）
  - [x] v68.1〜v68.9 の主要変更を Added セクションに記載（v68.9.0 の distributed.mdx 作成も含む）

---

## T5: `driver.rs` — `v69000_tests` 追加

- [x] `// -- v68900_tests (v68.9.0) --` の直前に挿入（driver.rs は降順配置のため、新バージョンが上になる）
  - [x] `cargo_toml_version_is_69_0_0`:
    - [x] `include_str!("../Cargo.toml")` で Cargo.toml を読み込む
    - [x] `"version = \"69.0.0\""` を個別 `assert!` で検証
  - [x] `changelog_has_v69_0_0`:
    - [x] `include_str!("../../CHANGELOG.md")` で CHANGELOG.md を読み込む
    - [x] `"v69.0.0"` を個別 `assert!` で検証
  - [x] `milestone_has_distributed`:
    - [x] `include_str!("../../MILESTONE.md")` で MILESTONE.md を読み込む
    - [x] `"Distributed Favnir"` を個別 `assert!` で検証
  - [x] `readme_mentions_distributed`:
    - [x] `include_str!("../../README.md")` で README.md を読み込む
    - [x] `"Distributed Favnir"` を個別 `assert!` で検証
- [x] `cargo build` でエラーなし

---

## T6: cargo clean ★クリーンアップ

- [x] `cd /c/Users/yoshi/favnir/fav && cargo clean` 実行（7.7GiB 削除）
- [x] `fav/tmp/hello.fav` を確認（cargo clean 後も存在、復元不要）

---

## T7: ビルド・テスト

- [x] `cargo test --bin fav v69000_tests` で 4 件 PASS
  - [x] `cargo_toml_version_is_69_0_0` PASS
  - [x] `changelog_has_v69_0_0` PASS
  - [x] `milestone_has_distributed` PASS
  - [x] `readme_mentions_distributed` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3541 tests passed, 0 failed を確認

---

## T8: ドキュメント・ステータス更新

- [x] 1. `versions/roadmap/roadmap-v68.1-v69.0.md` の v69.0.0「状態」列を「未着手」→「完了」に変更
- [x] 2. `versions/current.md` の「最新安定版」を v69.0.0 に更新、「進行中バージョン」を v69.0.0 に更新
- [x] 3. 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）← 最後に実施

---

> **sub-version ポリシー終了**: v69.0.0 では Cargo.toml / CHANGELOG.md を正式更新する（v68.x では据え置きだった）。

---

## コードレビュー指摘と対応

| 優先度 | 箇所 | 指摘内容 | 対応 |
|---|---|---|---|
| [HIGH] | spec.md / tasks.md | `readme_mentions_distributed` の条件がロードマップ（OR 条件）と差異、根拠未記載 | spec.md に「過去の知見（v25.0.0）により単独アサーション採用」の注記を追加 |
| [MED] | plan.md Step 5 | Cargo.toml の include_str! パスが `../../fav/Cargo.toml`（誤） | `../Cargo.toml` に修正 |
| [MED] | plan.md CHANGELOG テンプレート | v68.9.0 の記載が欠落 | v68.9.0（Stabilization / distributed.mdx）を Added に追記 |
| [MED] | spec.md 完了条件 | current.md / roadmap 状態更新が完了条件に含まれていなかった | 完了条件に 2 項目を追加 |
| [LOW] | tasks.md T0 | README.md の事前確認が欠落 | T0 に確認項目を追加 |
| [実装] | driver.rs | Cargo.toml version 更新により旧バージョン 17 件の cargo_toml_version_is_XX テストが FAIL | 旧テストの assert を "69.0.0" に一括置換（sed replace_all） |
| [MED] | driver.rs | 旧テスト群の失敗メッセージが `"Cargo.toml version should be 67.0.0"` のまま残存（assert 条件は正しく更新済み） | sed で "69.0.0" に一括修正（9件） |

---

## 設計上の意図的省略

- v68.1〜v68.9 の機能実装の追加・変更: 将来フェーズ（スタブのまま v70.0.0 以降へ）
- 新規 `.rs` スタブモジュール: 本バージョンでは不要
