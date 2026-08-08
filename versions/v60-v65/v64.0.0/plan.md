# v64.0.0 Plan — Incremental & Scale 宣言 ★クリーンアップ

Version: 64.0.0
Status: 未着手

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version を `"63.0.0"` → `"64.0.0"` に更新 |
| `MILESTONE.md` | v64.0.0 "Incremental & Scale" 宣言エントリを先頭に追加 |
| `README.md` | v64.0.0 宣言文を v63.0.0 エントリの前に追加 |
| `CHANGELOG.md` | v64.0.0 エントリを先頭に追加 |
| `fav/src/driver.rs` | `v64000_tests` を `v63900_tests` の直前に追加 |

---

## 実装ステップ

### Step 1: `fav/Cargo.toml` バージョン更新

`version = "63.0.0"` → `version = "64.0.0"`

### Step 2: `MILESTONE.md` 宣言エントリ追加

`## v63.0.0` エントリの前に v64.0.0 の宣言文・達成内容・テスト数を挿入。

### Step 3: `README.md` 追記

`v63.0.0` エントリの前に v64.0.0 の一行宣言を追加。

### Step 4: `CHANGELOG.md` エントリ追加

### Step 5: `driver.rs` — `v64000_tests` 追加

`v63900_tests` の直前に 4 テスト:
- `cargo_toml_version_is_64_0_0`（include_str!("../Cargo.toml") で version 確認）
- `changelog_has_v64_0_0`（include_str!("../../CHANGELOG.md") で v64.0.0 確認）
- `milestone_has_incremental_scale`（include_str!("../../MILESTONE.md") で宣言文確認）
- `readme_mentions_incremental_scale`（include_str!("../../README.md") で v64.0 確認）

### Step 6: ビルド・テスト確認

- `cargo build` エラーなし
- `cargo test --bin fav v64000_tests` で 4 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3431 tests passed, 0 failed

### Step 7: ★クリーンアップ

`cargo clean` 実行後、`fav/tmp/hello.fav` の存在を確認し、必要なら復元。
`cargo build` で再ビルド成功を確認。

---

## テスト計画

| テスト名 | 確認内容 |
|---|---|
| `cargo_toml_version_is_64_0_0` | Cargo.toml に `version = "64.0.0"` |
| `changelog_has_v64_0_0` | CHANGELOG.md に `v64.0.0` |
| `milestone_has_incremental_scale` | MILESTONE.md に `Incremental & Scale` |
| `readme_mentions_incremental_scale` | README.md に `Incremental & Scale` または `v64.0` |

ベース: 3427 → 目標: 3431（+4）
