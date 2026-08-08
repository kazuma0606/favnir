# Plan — v56.0.0 — Streaming Native 2.0 宣言 ★クリーンアップ

## ステップ

### Step 1: 事前確認

- `fav/Cargo.toml` が `55.9.0` であることを確認（更新前）
- `site/content/docs/streaming-native2-overview.mdx` が存在することを確認（v55.9.0 で追加済み）
- `driver.rs` の `v55900_tests` に `cargo_toml_version_is_55_9_0` が含まれることを確認（削除対象）
- `include_str!` パスの整合性を確認

---

### Step 2: `fav/Cargo.toml` バージョン更新

```toml
[package]
version = "56.0.0"
```

---

### Step 3: `MILESTONE.md` — v56.0.0 宣言文エントリ追加

`MILESTONE.md` の先頭に v56.0.0 エントリを追加する。
以下の宣言文（引用ブロック）を含む:

```
## v56.0.0（2026-07-24）— Streaming Native 2.0

> 「ウィンドウはイベントを時間で区切り、ウォーターマークは遅延を許容し、
>  チェックポイントは障害から瞬時に回復する。
>  CEP はイベントの流れからパターンを検出する。
>  Favnir はリアルタイムデータの言語になった。
>
>  これが Favnir v56.0 — Streaming Native 2.0 の姿である。」
```

---

### Step 4: `README.md` — Streaming Native 2.0 言及追加

マイルストーン一覧（最近バージョンの段落群）の先頭に v56.0 エントリを追加する。
`"Streaming Native 2.0"` を含む。

---

### Step 5: `CHANGELOG.md` — v56.0.0 エントリ追加

CHANGELOG.md 先頭（既存エントリの前）に追加する。
`"v56.0.0"` を含む。

---

### Step 6: `driver.rs` — `v56000_tests` 追加 + `cargo_toml_version_is_55_9_0` 削除

#### 6a. 削除

`v55900_tests` から `cargo_toml_version_is_55_9_0` 関数を削除する（1件減）。

#### 6b. 追加

`// -- v55900_tests` コメント行の直前に `v56000_tests` モジュールを挿入する（4件増）。

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

---

### Step 7: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

期待結果: `Finished`

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep -E "^test result|v56000|FAILED"
```

期待結果: `3227 tests passed, 0 failed`、v56000 4 件 ok

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -5
```

期待結果: クリーン

---

### Step 8: `cargo clean`（★クリーンアップ）

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

`fav/tmp/hello.fav` は `target/` ではなく `tmp/` にあるため clean 後も残る（確認のみ）。

---

### Step 9: ポスト処理

```
versions/current.md        → v56.0.0 / 3227 tests に更新
roadmap-v55.1-v56.0.md    → v56.0.0 実績を COMPLETE に更新
roadmap-v55.1-v60.0.md    → v56.0.0 実績欄も COMPLETE に更新
```

---

## テスト数の変化

| 操作 | 件数 |
|------|------|
| v55.9.0 完了時点ベース | 3224 |
| `cargo_toml_version_is_55_9_0` 削除 | -1 |
| `v56000_tests` 追加 | +4 |
| **合計（目標）** | **3227** |

---

## 注意事項

- `cargo_toml_version_is_55_9_0` の削除は **必須**。削除しないと 1 FAILED になる。
- `★クリーンアップ` は v56.0.0 の重要タスク（ロードマップ明記）。実施後はビルドキャッシュが削除されるが `fav/tmp/hello.fav` は影響なし。
- `include_str!` のパスは `fav/src/driver.rs` を起点とした相対パス。
