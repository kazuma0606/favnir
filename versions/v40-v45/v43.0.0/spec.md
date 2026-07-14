# v43.0.0 仕様書 — Real-Time Power 宣言 ★クリーンアップ

**フェーズ**: Real-Time Power（v42.x スプリント）
**前バージョン**: v42.9.0（v43.0 前調整・安定化、2896 tests）
**目標テスト数**: 2900（+4）

---

## 概要

v42.1〜v42.9 の Real-Time Power スプリントを「**Real-Time Power 宣言**」として正式に宣言するマイルストーンバージョン。

**宣言文:**

> 「CEP で `seq(Login, Purchase) within 300` が型安全に書ける。
>  Stream join で 2 ストリームを time-window で結合できる。
>  `#[max_inflight]` で Back-pressure を宣言的に制御できる。
>
>  これが Favnir v43.0 — Real-Time Power の姿である。」

Rust コードの機能追加はなし。ドキュメント・メタデータ整備と ★クリーンアップのみ。

---

## 現状確認

| ファイル | 状態 |
|---|---|
| `MILESTONE.md` | 先頭エントリは `v42.0.0 — Type Precision`。`Real-Time Power` は未掲載 |
| `README.md` | v42.0 の記述あり（line 110-111）。`Real-Time Power` は未掲載 |
| `fav/Cargo.toml` | version: `42.9.0` |
| `CHANGELOG.md` | `[v42.9.0]` が先頭エントリ |
| `fav/src/driver.rs` | `v42900_tests::cargo_toml_version_is_42_9_0` が NOTE コメント付きライブアサーション |
| `site/content/docs/real-time-power.mdx` | 存在（v42.9.0 で作成済み） |

---

## スコープ

### v43.0.0 に含む

1. **`MILESTONE.md` 更新** — `v43.0.0 — Real-Time Power` エントリを先頭に追加
2. **`README.md` 更新** — `Real-Time Power`（v43.0）の記述を v42.0 記述の直後に追加
3. **`fav/Cargo.toml`** — version: `42.9.0` → `43.0.0`
4. **`CHANGELOG.md`** — `[v43.0.0]` エントリを `[v42.9.0]` の直前に追加
5. **`fav/src/driver.rs`** — `v42900_tests` スタブ化 + `v43000_tests` 4 件追加
6. **`cargo test`** — 全通過（2900 tests, 0 failed）
7. **★`cargo clean`** — クリーンアップ実施 + `cargo test` 再通過確認

### スコープ外

- 新規言語機能
- `site/content/docs/real-time-power.mdx` 変更（v42.9.0 で作成済み）
- 新規 MDX ファイル作成

---

## 実装方針

### 1. `MILESTONE.md` 更新

v42.0.0 エントリの直前（先頭）に v43.0.0 エントリを追加:

```markdown
## v43.0.0 — Real-Time Power（2026-07-12）

> 「CEP で `seq(Login, Purchase) within 300` が型安全に書ける。
>  Stream join で 2 ストリームを time-window で結合できる。
>  `#[max_inflight]` で Back-pressure を宣言的に制御できる。
>
>  これが Favnir v43.0 — Real-Time Power の姿である。」

v43.0.0 をもって、Favnir の **Real-Time Power** を正式に宣言する。

### 達成コンポーネント（v42.1〜v42.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| CEP DSL 基盤 | v42.1 | `cep pattern` / `within` 構文 |
| CEP パターン: `seq` / `any` / `not` | v42.2 | 3 パターンコンビネータ |
| CEP checker.fav 統合 | v42.3 | `within >= 1` 検証・E0420 |
| Stream join（time-window） | v42.4 | `Stream.join` 2 ストリーム結合 |
| Back-pressure `#[max_inflight]` | v42.5 | parser + AST 宣言 |
| WebSocket Rune | v42.6 | `WebSocket.send` / `WebSocket.broadcast` |
| `fav monitor` | v42.7 | パイプライン監視コマンド stub |
| Real-Time Power cookbook | v42.8 | `cep-login-purchase.mdx` / `stream-join.mdx` |
| v43.0 前調整・安定化 | v42.9 | `real-time-power.mdx` 新規作成 |

**宣言日**: 2026-07-12
```

### 2. `README.md` 更新

v42.0 記述の直後（line 111 付近）に v43.0 の記述を追加:

```markdown
**v43.0（2026-07-12）で、[Real-Time Power](./MILESTONE.md) マイルストーンを宣言しました。**
CEP（`seq(Login, Purchase) within 300`）/ Stream join / Back-pressure / WebSocket Rune / fav monitor が揃い、サブ秒レイテンシのリアルタイムパイプラインを型安全に記述できる Real-Time Power 基盤が完成しました。
```

### 3. `driver.rs` テスト更新

#### 3a. `v42900_tests::cargo_toml_version_is_42_9_0` スタブ化

```rust
fn cargo_toml_version_is_42_9_0() {
    // Stubbed: version bumped to 43.0.0 -- assertion intentionally removed
    assert!(true);
}
```

#### 3b. `v43000_tests` モジュール追加（`v42900_tests` の直前）

```rust
// -- v43000_tests (v43.0.0) -- Real-Time Power 宣言 --
#[cfg(test)]
mod v43000_tests {
    #[test]
    fn cargo_toml_version_is_43_0_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("43.0.0"), "Cargo.toml must contain version 43.0.0");
    }

    #[test]
    fn changelog_has_v43_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v43.0.0]"), "CHANGELOG.md must contain [v43.0.0]");
    }

    #[test]
    fn milestone_has_real_time_power() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("Real-Time Power"), "MILESTONE.md must contain Real-Time Power");
    }

    #[test]
    fn readme_mentions_real_time_power() {
        let src = include_str!("../../README.md");
        assert!(src.contains("Real-Time Power"), "README.md must mention Real-Time Power");
    }
}
```

`v43000_tests` は `include_str!` のみ使用のため `use super::*` 不要。

---

## 既存コードへの影響

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `MILESTONE.md` | 変更 | v43.0.0 エントリ追加（先頭） |
| `README.md` | 変更 | `Real-Time Power` 記述 2 行追加 |
| `fav/Cargo.toml` | 変更 | version: `42.9.0` → `43.0.0` |
| `CHANGELOG.md` | 変更 | `[v43.0.0]` エントリ追加 |
| `fav/src/driver.rs` | 変更 | `v42900_tests` スタブ化 + `v43000_tests` 追加（4 件） |
| `versions/current.md` | 変更 | v43.0.0 最新安定版・v43.1.0 次版に更新 |
| `versions/roadmap/roadmap-v42.1-v43.0.md` | 変更 | v43.0.0 を完了済みにマーク |
| `versions/v40-v45/v43.0.0/tasks.md` | 変更 | COMPLETE ステータスに更新 |

Rust ソースコード変更なし（宣言・クリーンアップのみ）。

---

## テスト計画

### Rust テスト（driver.rs）— 4 件

```rust
mod v43000_tests {
    fn cargo_toml_version_is_43_0_0()   // Cargo.toml に "43.0.0" が含まれる
    fn changelog_has_v43_0_0()          // CHANGELOG.md に "[v43.0.0]" が含まれる
    fn milestone_has_real_time_power()  // MILESTONE.md に "Real-Time Power" が含まれる
    fn readme_mentions_real_time_power() // README.md に "Real-Time Power" が含まれる
}
```

テスト数: 2896 + 4 = **2900**

---

## ★cargo clean 手順

1. `cargo test` 全通過（2900 tests, 0 failed）を確認
2. `cargo clean` を実行
3. `fav/tmp/hello.fav` の存在を確認（消えた場合は復元）
   - 復元内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`
4. `cargo test` を再実行し 2900 passed / 0 failed を確認

---

## 完了条件

- `cargo test` 全通過（2900 tests passed, 0 failed）
- `v43000_tests::cargo_toml_version_is_43_0_0` pass
- `v43000_tests::changelog_has_v43_0_0` pass
- `v43000_tests::milestone_has_real_time_power` pass
- `v43000_tests::readme_mentions_real_time_power` pass
- `MILESTONE.md` の先頭に `v43.0.0 — Real-Time Power` エントリが存在する
- `README.md` に `Real-Time Power` の記述が含まれる
- `versions/current.md` が v43.0.0 最新安定版・v43.1.0 次版に更新されている
- `cargo clean` 完了・`cargo test` 再通過確認

---

## 非スコープ

- 新規言語機能・VM 機能（コードフリーズ）
- `fav monitor` 実測メトリクス実装 — v43.x 以降
- `#[max_inflight]` runtime back-pressure — v44.x 以降
- WebSocket 実接続 — v44.x 以降
