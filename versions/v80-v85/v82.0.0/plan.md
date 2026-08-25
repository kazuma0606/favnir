# v82.0.0 実装計画

## 方針

宣言バージョン。新規 API / struct の追加なし。
クリーンアップ + ドキュメント更新 + v82000_tests 4 件の追加。

**前提**: v81.9.0 完了済み（3,861 tests pass）。

---

## 実装ステップ

### Step 1: `cargo clean`

```bash
cargo clean
```

ビルドキャッシュを削除し、クリーンな状態でビルドを確認する。

### Step 2: `Cargo.toml` バージョン更新

`fav/Cargo.toml` の `version` を `"81.0.0"` → `"82.0.0"` に変更する。

> **注意**: Cargo.toml のバージョンは宣言版（v81.0.0, v82.0.0 等）でのみ更新される。
> v81.x.x の各スプリントでは更新しないため、現在の値は依然 `"81.0.0"` のままである。

### Step 3: CHANGELOG 更新

`CHANGELOG.md` の先頭に v82.0.0 エントリを追加する（宣言文・達成内容・sprint 一覧）。

### Step 4: MILESTONE.md 更新

`MILESTONE.md` の先頭に Data Quality 2.0 宣言エントリを追加する。
v81.1〜v81.9 の達成内容を列挙する。

### Step 5: README.md 更新

`README.md` の先頭バージョンセクションを v82.0 に更新する。
`QualityGate` / `QualityScore` / `SchemaDriftDetector` / `AnomalyDetector` に言及する。

### Step 6: `versions/current.md` 更新

- 最新安定版を `v82.0.0` に更新
- 進行中バージョンを `v82.1.0〜v83.0.0`（Pipeline Contracts 1.0 スプリント）に更新
- マイルストーン進捗テーブルに `v82.0` を完了として追記

### Step 7: ロードマップ更新

- `roadmap-v80.1-v85.0.md`: Sprint 2 テーブルの v82.0.0 行を「完了」に更新
- `roadmap-v81.1-v82.0.md`: 全バージョン行のステータスを「完了」に更新する（事前に `versions/current.md` が `roadmap-v80.1-v85.0.md` を指していることを確認）

### Step 8: `v82000_tests` テストモジュール追加

`fav/src/driver.rs` の末尾に `#[cfg(test)] mod v82000_tests` を追加する。
4 件のテスト:
- `cargo_toml_version_is_82_0_0`
- `changelog_has_v82_0_0`
- `milestone_has_data_quality_2`
- `readme_mentions_quality_gate`

### Step 9: `cargo test` 全通過確認

```bash
cargo test 2>&1 | grep "test result"
```

3,865 tests pass（+4）、0 failures であることを確認する。
