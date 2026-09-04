# Plan: v100.0.0 — Favnir SAP Platform 1.0 宣言 ★大クリーンアップ

## 実装順序

### Step 1: fav/Cargo.toml version を 100.0.0 に更新

`version = "99.0.0"` → `version = "100.0.0"` に変更する。
この変更が `cargo_toml_version_is_100_0_0` テストの前提となる。

### Step 2: MILESTONE.md に v100.0.0 エントリを追加

`MILESTONE.md` のマイルストーン一覧に以下を追加する:

```
| v100.0 — SAP Platform 1.0 | **完了** | v99.1〜v99.9 完了後（2026-09-04）|
```

`SAP Platform` キーワードが含まれること（`milestone_has_sap_platform` テストの前提）。

### Step 3: README.md に SAP Platform 1.0 セクションを追加

`README.md` に `## v100.0 — Favnir SAP Platform 1.0` セクションを追加し、
宣言文の要旨と主要機能一覧を記載する。
`SAP Platform` キーワードが含まれること（`readme_mentions_sap_platform` テストの前提）。

### Step 4: CHANGELOG.md に [v100.0.0] エントリを追加

CHANGELOG.md の先頭に `[v100.0.0]` エントリを追加する。
`[v100.0.0]` キーワードが含まれること（`changelog_has_v100_0_0` テストの前提）。

**注意**: Step 4 は Step 5（driver.rs にテスト追加）より**前に必ず完了すること**。
`changelog_has_v100_0_0` テストが CHANGELOG の更新を前提とするため。

### Step 5: driver.rs に mod v100000_tests を追加

`fav/src/driver.rs` の `mod v99900_tests` 直後に `mod v100000_tests`（4 テスト）を追加する。

- `cargo_toml_version_is_100_0_0`: `include_str!("../Cargo.toml")` に `"100.0.0"` が含まれることを確認
- `changelog_has_v100_0_0`: `../CHANGELOG.md` に `"[v100.0.0]"` が含まれることを確認
- `milestone_has_sap_platform`: `../MILESTONE.md` に `"SAP Platform"` が含まれることを確認
- `readme_mentions_sap_platform`: `../README.md` に `"SAP Platform"` が含まれることを確認

ブロック先頭に `// use super::* は不要（外部シンボル未使用）` コメントを記載する。

### Step 6: cargo test で全 pass 確認（大クリーンアップ前）

```
cargo test -- --test-threads=1 2>&1 | grep "test result"
```

4,279 tests, 0 failures を確認する。

### Step 7: ★大クリーンアップ（cargo clean → cargo test 再確認）

1. `cargo clean` を実行する（target/ ディレクトリを削除）
2. `fav/tmp/hello.fav` が削除されていないことを確認する
3. もし `hello.fav` が消えていた場合は復元する（内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`）
4. `cargo test -- --test-threads=1 2>&1 | grep "test result"` で 4,279 tests, 0 failures を再確認する
5. `cargo build` で `./target/debug/fav` を再生成する

### Step 8: CI チェック（cargo clean 後にビルド済みであることが前提）

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`

### Step 9: versions/current.md を v100.0.0 に更新

### Step 10: SAP ロードマップファイルの Status を「完了」に更新

以下のファイルを「完了」に更新する:
1. `versions/roadmap/roadmap-v99.1-v100.0.md` — v100.0.0 セクションまたはファイル冒頭の Status 行を「完了」に更新する
2. `versions/roadmap/roadmap-v95.1-v100.0.md` — ファイル冒頭の Status 行を「完了」に更新する（存在する場合）

## 依存関係

- Step 1 → Step 5（cargo_toml_version_is_100_0_0 の前提）
- Step 2 → Step 5（milestone_has_sap_platform の前提）
- Step 3 → Step 5（readme_mentions_sap_platform の前提）
- Step 4 → Step 5（changelog_has_v100_0_0 の前提、かつ順序必須）
- Step 5 → Step 6
- Step 6 → Step 7（cargo clean は最初の test pass 後）
- Step 7 → Step 8
- Step 8 → Step 9, Step 10（並列可）

**注意**: Step 7 の `cargo clean` 後は `cargo build` で `./target/debug/fav` を再生成してから Step 8（CI チェック）を実施すること。
