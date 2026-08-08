# v66.0.0 Spec — Math & Science Foundation 宣言 ★クリーンアップ

Version: 66.0.0
Status: 未着手
Base tests: 3471
Target tests: 3475

---

## 概要

v65.1〜v65.9 で実装した Math & Science Rune 群を正式宣言するマイルストーンバージョン。
`cargo clean` による成果物クリーンアップを行い、フルビルド・全テスト通過を確認する。

**宣言文**:

> 「行列の次元は型で保証され、勾配は自動的に伝播する。
>  統計的検定は型安全に呼び出せ、時系列の周期は型パラメータに刻まれる。
>  数学的正確性が、AI パイプラインの信頼性を支える土台になった。
>
>  これが Favnir v66.0 — Math & Science Foundation の姿である。」

ロードマップ `roadmap-v65.1-v66.0.md` の v66.0.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test --bin fav` でベース 3471 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（本バージョンで `"66.0.0"` に更新する）
- `driver.rs` に `v65900_tests` が存在することを確認（`v66000_tests` の挿入位置）
- `driver.rs` に `v66000_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v65900_tests` で 2 件 PASS することを確認（前バージョンが正常）
- `versions/current.md` の「進行中バージョン」が `v65.9.0` であることを確認

---

## 実装スコープ

### 1. `fav/Cargo.toml` — バージョン更新

```toml
version = "66.0.0"
```

（`"65.0.0"` → `"66.0.0"` に変更）

### 2. `MILESTONE.md` — v66.0.0 エントリを先頭に追加

```markdown
## v66.0.0（2026-08-05）— Math & Science Foundation

> 「行列の次元は型で保証され、勾配は自動的に伝播する。
>  統計的検定は型安全に呼び出せ、時系列の周期は型パラメータに刻まれる。
>  数学的正確性が、AI パイプラインの信頼性を支える土台になった。
>
>  これが Favnir v66.0 — Math & Science Foundation の姿である。」

**Math & Science Foundation** の宣言バージョン。v65.1〜v65.9 で実装した
線形代数・統計・自動微分・最適化・数値計算・時系列・ML Primitives の 7 Rune 群と
Math Lint Rules W050〜W054 の統合を宣言した。

**v65.1〜v65.9 達成内容:**
- v65.1（Rune.linalg）: 線形代数（matmul / svd / eig / solve）
- v65.2（Rune.stats）: 統計解析（describe / t_test / linear_regression）
- v65.3（Rune.autodiff）: 自動微分（grad / jacobian / hessian / tape）
- v65.4（Rune.optim）: 最適化（adam / sgd / l_bfgs / scheduler）
- v65.5（Rune.numeric）: 数値計算（integrate / fft / ode_solve / bisection）
- v65.6（Rune.timeseries）: 時系列（arima / sarima / decompose / adf_test）
- v65.7（Rune.ml）: ML Primitives（knn / random_forest / cross_validate）
- v65.8（Math Lint Rules）: W050〜W054 静的解析ルール
- v65.9（安定化）: math-runes-overview.mdx / 全 Rune 存在確認

**テスト数**: 3475

---
```

（既存の `## v65.0.0` エントリの直前に挿入）

### 3. `README.md` — v66.0.0 宣言を追加

既存の v65.0.0 / Performance 1.0 の記述の直前に v66.0.0 の言及を追加。
`"Math & Science"` または `"v66.0"` を含む必要がある（`readme_mentions_math_science` テストで検証）。

### 4. `CHANGELOG.md` — v66.0.0 エントリを先頭に追加

```markdown
## [v66.0.0] — 2026-08-05 — Math & Science Foundation 宣言 ★クリーンアップ

### Added
- `MILESTONE.md` に v66.0.0「Math & Science Foundation」宣言文エントリを追加
- `v66000_tests`: 4 件追加（3471 → 3475 tests）
  - `cargo_toml_version_is_66_0_0`
  - `changelog_has_v66_0_0`
  - `milestone_has_math_science`
  - `readme_mentions_math_science`
- `site/content/docs/runes/math-runes-overview.mdx` 新規作成（v65.9.0）
- Math & Science Rune 群（v65.1〜v65.9）の成果を統合:
  - `Rune.linalg`（v65.1）/ `Rune.stats`（v65.2）/ `Rune.autodiff`（v65.3）
  - `Rune.optim`（v65.4）/ `Rune.numeric`（v65.5）/ `Rune.timeseries`（v65.6）
  - `Rune.ml`（v65.7）/ Math Lint Rules W050〜W054（v65.8）

### Changed
- `fav/Cargo.toml` version `"65.0.0"` → `"66.0.0"`
- `README.md` に Math & Science Foundation 宣言を追記

### Note
- ★クリーンアップ（`cargo clean`）完了
- `cargo clean` 後は `fav/tmp/hello.fav` を復元すること（bootstrap テスト要件）

---
```

（既存の `## [v65.0.0]` エントリの直前に挿入）

### 5. `driver.rs` — `v66000_tests` 追加

挿入位置: `// -- v65900_tests (v65.9.0)` コメントの直前

```rust
// -- v66000_tests (v66.0.0) -- Math & Science Foundation 宣言 --
#[cfg(test)]
mod v66000_tests {
    #[test]
    fn cargo_toml_version_is_66_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(
            toml.contains("version = \"66.0.0\""),
            "Cargo.toml should have version 66.0.0: {}",
            &toml[..200.min(toml.len())]
        );
    }

    #[test]
    fn changelog_has_v66_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("v66.0.0"), "CHANGELOG.md should mention v66.0.0");
    }

    #[test]
    fn milestone_has_math_science() {
        let ms = include_str!("../../MILESTONE.md");
        assert!(
            ms.contains("Math & Science"),
            "MILESTONE.md should contain 'Math & Science'"
        );
    }

    #[test]
    fn readme_mentions_math_science() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Math & Science") || readme.contains("v66.0"),
            "README.md should mention Math & Science Foundation or v66.0"
        );
    }
}
```

### 6. `cargo clean` + `fav/tmp/hello.fav` 復元

★クリーンアップとして `cargo clean` を実行。
**実行後、`fav/tmp/hello.fav` を必ず復元すること**（削除されると `bootstrap_c2_artifact_roundtrip` テストが FAIL する）。

`fav/tmp/hello.fav` の正しい内容:
```favnir
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

---

## 完了条件

- `fav/Cargo.toml` に `version = "66.0.0"` が含まれる
- `CHANGELOG.md` に `"v66.0.0"` が含まれる
- `MILESTONE.md` に `"Math & Science"` が含まれる
- `README.md` に `"Math & Science"` または `"v66.0"` が含まれる
- `cargo test --bin fav v66000_tests` で 4 件 PASS
  - `cargo_toml_version_is_66_0_0` PASS
  - `changelog_has_v66_0_0` PASS
  - `milestone_has_math_science` PASS
  - `readme_mentions_math_science` PASS
- `cargo clean` 実行済み
- `fav/tmp/hello.fav` 復元済み
- `cargo test -j 8 -- --test-threads=8` で 3475 tests passed, 0 failed

---

## 非スコープ

- 各 Rune の型システム登録 — 将来フェーズ
- W050〜W054 の実際の検出ロジック — 将来フェーズ
- v67.x 以降のスプリント計画 — 別途策定

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../Cargo.toml"` → `fav/Cargo.toml`（1つ上の `fav/` ディレクトリ）
- `"../../CHANGELOG.md"` → `favnir/CHANGELOG.md`（リポジトリルート）
- `"../../MILESTONE.md"` → `favnir/MILESTONE.md`（リポジトリルート）
- `"../../README.md"` → `favnir/README.md`（リポジトリルート）

### `cargo clean` 後の `hello.fav` 復元理由

`cargo clean` は `target/` を削除するが、`fav/tmp/` 以下のファイルも一部消去される場合がある（ビルドスクリプト依存）。
`bootstrap_c2_artifact_roundtrip` テストが `fav/tmp/hello.fav` を参照するため、削除後は手動復元が必要。

### テスト数の変化（+4）

マイルストーン宣言バージョン（x.0.0 リリース）では通常 +4 テスト。
サブバージョン（x.y.0）の +2 とは異なる。

### `readme_mentions_math_science` の OR 条件

`contains("Math & Science") || contains("v66.0")` を使用。
README には将来 v66.0 以外の記述でマッチできる柔軟性を持たせる。
