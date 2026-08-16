# v76.0.0 仕様書 — Temporal Data Native 宣言 ★クリーンアップ

Date: 2026-08-15
Status: 計画中

---

## Background

v75.1.0〜v75.9.0 で実装した Temporal Data Native スプリント（FreshnessPolicy、TemporalRange/AsOfQuery、SCD 2.0、TemporalJoinConfig、RetentionPolicy、StreamFreshnessMonitor、TemporalContract、TimeTravelQuery）が安定化・コードフリーズを経て完成した。v76.0.0 ではマイルストーン宣言を行い、MILESTONE.md / README.md を更新し、`cargo clean` によりビルドキャッシュをリセットする。

---

## 宣言文

> 「鮮度が型となり、SCD が構造となり、タイムトラベルが API となった。
>  Favnir のパイプラインは今、時間軸を型で保証する。」

---

## Goals

1. `MILESTONE.md` の先頭に v76.0.0 Temporal Data Native 宣言エントリを追加する
2. `README.md` に v76.0 Temporal Data Native 宣言セクションを追加する
3. `CHANGELOG.md` に v76.0.0 エントリを追加する
4. `v76000_tests` を実装する（4 件）
5. `cargo clean` を実施する（★クリーンアップ）
6. `cargo test` で 3714 tests pass を確認する

---

## テスト仕様（v76000_tests）

### `cargo_toml_version_is_76_0_0`

```rust
let content = include_str!("../Cargo.toml");
assert!(content.contains("version = \"76.0.0\""));
```

### `changelog_has_v76_0_0`

```rust
let content = include_str!("../../CHANGELOG.md");
assert!(content.contains("[v76.0.0]"));
```

### `milestone_has_temporal_data_native`

```rust
let content = include_str!("../../MILESTONE.md");
assert!(content.contains("Temporal Data Native"));
```

### `readme_mentions_temporal`

```rust
let content = include_str!("../../README.md");
assert!(content.contains("Temporal"));
```

---

## Success Criteria

- `cargo_toml_version_is_76_0_0` が pass
- `changelog_has_v76_0_0` が pass
- `milestone_has_temporal_data_native` が pass
- `readme_mentions_temporal` が pass
- `cargo test` が 3714 tests all pass
- `CHANGELOG.md` の先頭に `[v76.0.0]` エントリが存在する
- `MILESTONE.md` の先頭に v76.0.0 エントリが存在する
- `README.md` に "Temporal" の記述が存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `v76000_tests` を追加
- `CHANGELOG.md` — v76.0.0 エントリを先頭に追加
- `MILESTONE.md` — v76.0.0 Temporal Data Native 宣言を先頭に追加
- `README.md` — v76.0 宣言セクションを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `75.9.0` → `76.0.0` に更新
- `fav/Cargo.lock` — バージョン更新に伴い自動更新

---

## ★クリーンアップ手順

`cargo clean` を実施した後、`fav/tmp/hello.fav` を復元してから `cargo test` を実行する。

`fav/tmp/hello.fav` の正しい内容:
```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```
