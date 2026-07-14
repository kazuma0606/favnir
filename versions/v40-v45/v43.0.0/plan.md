# v43.0.0 実装計画 — Real-Time Power 宣言 ★クリーンアップ

## 目標

v42.1〜v42.9 の Real-Time Power スプリントを正式宣言する。
新規機能追加なし。MILESTONE.md / README.md / CHANGELOG.md / driver.rs のメタデータ整備と `cargo clean` のみ。

---

## T0 — 事前確認

- [ ] `cargo test` が 2896 tests / 0 failures であることを確認
- [ ] `fav/Cargo.toml` version が `42.9.0` であることを確認
- [ ] `v42900_tests::cargo_toml_version_is_42_9_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録
- [ ] NOTE コメントが欠落している場合は実装を中断し報告すること
- [ ] `v42900_tests` の閉じ `}` の行番号を確認し記録
- [ ] `MILESTONE.md` に `Real-Time Power` が含まれないことを確認
- [ ] `README.md` に `Real-Time Power` が含まれないことを確認
- [ ] `driver.rs` に `v43000_tests` モジュールが存在しないことを確認

---

## T1 — `MILESTONE.md` 更新

v42.0.0 エントリの直前（先頭）に v43.0.0 エントリを追加:

```markdown
## v43.0.0 — Real-Time Power（2026-07-12）

> 「CEP で `seq(Login, Purchase) within 300` が型安全に書ける。
>  Stream join で 2 ストリームを time-window で結合できる。
>  `@max_inflight` で Back-pressure を宣言的に制御できる。
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

---
```

---

## T2 — `README.md` 更新

v42.0 記述（`**v42.0（2026-07-12）で...`）の直後に追加:

```markdown
**v43.0（2026-07-12）で、[Real-Time Power](./MILESTONE.md) マイルストーンを宣言しました。**
CEP（`seq(Login, Purchase) within 300`）/ Stream join / Back-pressure / WebSocket Rune / fav monitor が揃い、サブ秒レイテンシのリアルタイムパイプラインを型安全に記述できる Real-Time Power 基盤が完成しました。
```

---

## T3 — `fav/Cargo.toml` バージョン bump

`version = "42.9.0"` → `"43.0.0"`

---

## T4 — `CHANGELOG.md` 更新

`[v42.9.0]` の直前に `[v43.0.0]` エントリを追加:

```markdown
## [v43.0.0] — 2026-07-12

### Added
- `v43000_tests`: `cargo_toml_version_is_43_0_0` / `changelog_has_v43_0_0` / `milestone_has_real_time_power` / `readme_mentions_real_time_power`
- `MILESTONE.md` に `v43.0.0 — Real-Time Power` エントリを追加

### Changed
- `README.md` に Real-Time Power（v43.0）の記述を追加
- `fav/Cargo.toml` version: `42.9.0` → `43.0.0`
- `v42900_tests::cargo_toml_version_is_42_9_0` をスタブ化

### Notes
- Real-Time Power 宣言（v42.1〜v42.9 スプリント完了）
- `cargo clean` ★クリーンアップ実施

---
```

---

## T5 — `driver.rs` テストモジュール更新

#### 5a. `v42900_tests::cargo_toml_version_is_42_9_0` スタブ化

```rust
fn cargo_toml_version_is_42_9_0() {
    // Stubbed: version bumped to 43.0.0 -- assertion intentionally removed
    assert!(true);
}
```

#### 5b. `v43000_tests` モジュール追加（`v42900_tests` の直前）

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

---

## T6 — テスト実行・確認（クリーンアップ前）

- [ ] `cargo test` 実行
- [ ] failures = 0 を確認
- [ ] テスト数 = 2900 を確認（2896 + 4 件）
- [ ] `v43000_tests` 4 件すべて pass を確認
- [ ] 既存テストが壊れていないことを確認

---

## T7 — ★cargo clean + hello.fav 確認 + cargo test 再実行

- [ ] `cargo clean` を実行
- [ ] `fav/tmp/hello.fav` の存在を確認（消えた場合は以下で復元）
  ```
  fn add(a: Int, b: Int) -> Int { a + b }
  fn main() -> Bool { add(1, 2) == 3 }
  ```
- [ ] `cargo test` を再実行し 2900 passed / 0 failed を確認

---

## T8 — バージョン管理ドキュメント更新

- [ ] `versions/current.md` を v43.0.0（最新安定版）・v43.1.0（次に切る版）に更新
- [ ] `versions/roadmap/roadmap-v42.1-v43.0.md` の v43.0.0 を完了済みにマーク（`✅ COMPLETE（2026-07-12）`）
- [ ] `versions/v40-v45/v43.0.0/tasks.md` を COMPLETE ステータスに更新（全チェックボックス `[x]`）

---

## ファイル変更サマリー

| ファイル | 変更種別 |
|---|---|
| `MILESTONE.md` | 変更（先頭にエントリ追加） |
| `README.md` | 変更（Real-Time Power 記述追加） |
| `fav/Cargo.toml` | 変更（version bump） |
| `CHANGELOG.md` | 変更（エントリ追加） |
| `fav/src/driver.rs` | 変更（スタブ化 + v43000_tests 追加） |
| `versions/current.md` | 変更 |
| `versions/roadmap/roadmap-v42.1-v43.0.md` | 変更 |
| `versions/v40-v45/v43.0.0/tasks.md` | 変更 |
