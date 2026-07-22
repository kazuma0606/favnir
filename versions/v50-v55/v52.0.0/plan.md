# Plan: v52.0.0 — Performance & Scale 宣言

Date: 2026-07-20

---

## 実装順序

### Step 1 — 事前確認

- `cargo test` 3133 passed, 0 failed を確認（ベース確認）
- `cargo clippy -- -D warnings` クリーンであることを確認
- `MILESTONE.md` に `"Performance & Scale"` が**存在しない**ことを確認（新規追加対象）
- `README.md` に `"Performance & Scale"` が**存在しない**ことを確認（新規追加対象）
- `include_str!` パスの確認:
  - `../Cargo.toml` → `fav/Cargo.toml`
  - `../../CHANGELOG.md` → `favnir/CHANGELOG.md`
  - `../../MILESTONE.md` → `favnir/MILESTONE.md`
  - `../../README.md` → `favnir/README.md`
- `v51900_tests` に `cargo_toml_version_is_51_9_0` が存在することを確認（削除対象）

---

### Step 2 — `MILESTONE.md` 更新

先頭に v52.0.0 エントリを追加する。

```markdown
## v52.0.0（2026-07-20）— Performance & Scale

> 「並列パイプラインはコアを使い切り、バックプレッシャーは
>  データの氾濫を防ぎ、ベンチマークは退行を即座に検出する。
>  Favnir は大規模データに立ち向かえる言語になった。
>
>  これが Favnir v52.0 — Performance & Scale の姿である。」

**Performance & Scale** の宣言バージョン。v51.1〜v51.9 の全機能統合を経て、
並列実行・バックプレッシャー・ベンチマーク回帰検出・WASM 最適化の成熟を宣言する。

---
```

---

### Step 3 — `README.md` 更新

既存のマイルストーン言及箇所（`v34.0（2026-07-04）で、Performance & Tooling` 付近）に
v52.0.0「Performance & Scale」への言及を追加する。

---

### Step 4 — `CHANGELOG.md` 更新

先頭に v52.0.0 エントリを追加する。

---

### Step 5 — `v52000_tests` 追加 + バージョン更新

`driver.rs` に `v52000_tests` モジュールを `v51900_tests` 直前に追加（4 件）。
`v51900_tests` から `cargo_toml_version_is_51_9_0` を削除。
`Cargo.toml` を `"52.0.0"` に更新。

---

### Step 6 — ★クリーンアップ（`cargo clean`）

```bash
cd favnir/fav && cargo clean
```

その後 `cargo test` で clean state から全通過を確認。
`cargo clippy -- -D warnings` もクリーンであることを確認。

---

### Step 7 — 後処理

- `versions/current.md` を v52.0.0（3136 tests）に更新
- `roadmap-v51.1-v52.0.md` の v52.0.0 実績欄を更新
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `MILESTONE.md` | v52.0.0 エントリ追加 |
| `README.md` | Performance & Scale 言及追加 |
| `CHANGELOG.md` | v52.0.0 エントリ追加 |
| `fav/src/driver.rs` | `v52000_tests` 追加、`cargo_toml_version_is_51_9_0` 削除 |
| `fav/Cargo.toml` | version → `"52.0.0"` |
| `fav/Cargo.lock` | 自動更新 |
| `versions/current.md` | v52.0.0 に更新 |
| `versions/roadmap/roadmap-v51.1-v52.0.md` | v52.0.0 実績欄更新 |
