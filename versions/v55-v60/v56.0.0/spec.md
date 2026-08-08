# Spec — v56.0.0 — Streaming Native 2.0 宣言 ★クリーンアップ

## 概要

v56.0.0 は Streaming Native 2.0 スプリント（v55.1〜v55.9）の完結宣言バージョン。
v55.1〜v55.9 で積み上げた全ストリーミング機能を統合し、Streaming Native 2.0 として正式宣言する。
宣言文を `MILESTONE.md` に追記し、`README.md` に言及を加え、`CHANGELOG.md` にエントリを追加する。
`driver.rs` に `v56000_tests`（4 件）を追加し、最後に `cargo clean` でビルドキャッシュを完全削除する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v55.1-v56.0.md` — v56.0.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v56.0.0 行
- ベーステスト数: **3224**（v55.9.0 完了時点の実績値）
- 目標テスト数: **3227**（-1 + 4 = +3）

> **ロードマップ目標値（3228）との差異について**:
> ロードマップには「ベース 3224 + 4 = 3228」と記載されているが、
> v55900_tests の `cargo_toml_version_is_55_9_0` が v56.0.0 バージョン更新後に FAIL するため削除が必要（-1）。
> 削除 1 件 + 追加 4 件 = 実質 +3 件。実績は **3227** tests passed（3224 - 1 + 4 = 3227）。

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| タンブリング / スライディングウィンドウ + Exactly-once 統合 | v55.1.0 | 実装済み |
| セッションウィンドウ + ウォーターマーク | v55.2.0 | 実装済み |
| Exactly-once チェックポイント | v55.3.0 | 実装済み |
| ストリーム結合（join_inner / join_left） | v55.4.0 | 実装済み |
| Stateful stage（State API） | v55.5.0 | 実装済み |
| CEP 統合（sequence / skip_until） | v55.6.0 | 実装済み |
| Checkpoint / Replay API | v55.7.0 | 実装済み |
| Streaming 2.0 ドキュメント（MDX 3 件） | v55.8.0 | 実装済み |
| `streaming-native2-overview.mdx` 骨子 | v55.9.0 | 実装済み |
| Streaming Native 2.0 正式宣言 | v56.0.0 | **本バージョン** |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "56.0.0"
```

---

### 2. `MILESTONE.md` — v56.0.0 宣言文エントリ追加

以下の宣言文（引用ブロック）を先頭に追加する:

```
## v56.0.0（2026-07-24）— Streaming Native 2.0

> 「ウィンドウはイベントを時間で区切り、ウォーターマークは遅延を許容し、
>  チェックポイントは障害から瞬時に回復する。
>  CEP はイベントの流れからパターンを検出する。
>  Favnir はリアルタイムデータの言語になった。
>
>  これが Favnir v56.0 — Streaming Native 2.0 の姿である。」
```

必須キーワード: `"Streaming Native 2.0"`

---

### 3. `README.md` — Streaming Native 2.0 マイルストーン追記

マイルストーン一覧の先頭（最新版として）に v56.0.0 エントリを追加する。
`"Streaming Native 2.0"` キーワードを含む。

---

### 4. `CHANGELOG.md` — v56.0.0 エントリ追加

CHANGELOG.md 先頭に以下を追加する:

```markdown
## [v56.0.0] — 2026-07-24 — Streaming Native 2.0 宣言 ★クリーンアップ

### Added
- `MILESTONE.md` に v56.0.0 — Streaming Native 2.0 宣言文エントリを追加
- `v56000_tests` 追加（4 件）— 3227 tests
- `cargo clean` 実施（ビルドキャッシュ完全削除）
```

必須キーワード: `"v56.0.0"`

---

### 5. `fav/src/driver.rs` — `v56000_tests` モジュール追加 + `cargo_toml_version_is_55_9_0` 削除

#### 5a. v55900_tests から削除するテスト

`cargo_toml_version_is_55_9_0` — Cargo.toml が "56.0.0" になるため FAIL する。削除必須。

#### 5b. v55900_tests の直前（`// -- v55900_tests` コメント行の前）に挿入

```rust
// -- v56000_tests (v56.0.0) -- Streaming Native 2.0 宣言 ★クリーンアップ --
#[cfg(test)]
mod v56000_tests {
    #[test]
    fn cargo_toml_version_is_56_0_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"56.0.0\""),
            "Cargo.toml version should be 56.0.0, got: {}",
            cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
        );
    }

    #[test]
    fn changelog_has_v56_0_0() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(
            changelog.contains("v56.0.0"),
            "CHANGELOG.md must contain v56.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_streaming_native2() {
        let milestone = include_str!("../../MILESTONE.md");
        assert!(
            milestone.contains("Streaming Native 2.0"),
            "MILESTONE.md must mention Streaming Native 2.0"
        );
    }

    #[test]
    fn readme_mentions_streaming_native2() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Streaming Native 2.0"),
            "README.md must mention Streaming Native 2.0"
        );
    }
}
```

> **`include_str!` パス（`fav/src/driver.rs` 起点）**:
> - `../Cargo.toml` → `fav/Cargo.toml`
> - `../../CHANGELOG.md` → `favnir/CHANGELOG.md`
> - `../../MILESTONE.md` → `favnir/MILESTONE.md`
> - `../../README.md` → `favnir/README.md`

---

### 6. `cargo clean`（★クリーンアップ）

```bash
cd fav && cargo clean
```

ビルドキャッシュを完全削除する。`fav/tmp/hello.fav` は `target/` ではなく `tmp/` にあるため影響なし。

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `cargo_toml_version_is_56_0_0` | `fav/Cargo.toml` に `version = "56.0.0"` が含まれる |
| `changelog_has_v56_0_0` | `CHANGELOG.md` に `v56.0.0` が含まれる |
| `milestone_has_streaming_native2` | `MILESTONE.md` に `Streaming Native 2.0` が含まれる |
| `readme_mentions_streaming_native2` | `README.md` に `Streaming Native 2.0` が含まれる |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3227 tests passed, 0 failed**）— `v55100_tests`〜`v55900_tests` が全 pass であることを含む
- `cargo clippy -- -D warnings` クリーン
- `v56000_tests` 4 件すべて pass
- `MILESTONE.md` に `"Streaming Native 2.0"` 宣言文エントリが含まれる
- `CHANGELOG.md` に `v56.0.0` エントリが追加されている
- `cargo clean` 完了
- `versions/current.md` が v56.0.0 / 3227 tests を反映
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v56.0.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.0.0 実績欄も COMPLETE に更新

---

## 備考

- ロードマップ目標テスト数（3228）と実績（3227）が 1 件ずれる理由:
  v55900_tests の `cargo_toml_version_is_55_9_0` は v56.0 バージョン更新後に必ず FAIL するため削除（-1）。
  v56000_tests 4 件追加（+4）。3224 - 1 + 4 = **3227**。
- `★クリーンアップ` は v56.0.0 の重要な実施事項（33GB のビルドキャッシュ削除）。
- v56.x スプリントで Replay 完全実装・`RESUME_FROM_CHECKPOINT` の VM exec ループ統合を行う予定。
