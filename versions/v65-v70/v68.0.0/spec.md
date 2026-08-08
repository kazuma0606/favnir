# v68.0.0 Spec — Developer Intelligence 宣言 ★クリーンアップ

Version: 68.0.0
Status: 未着手
Base tests: 3515
Target tests: 3519

---

## 概要

v67.1〜v67.9 で実装した Developer Intelligence ツール群（デバッガ・DAG 可視化・AI 提案・合成テスト・Property Testing・インタラクティブプロファイリング・数式ドキュメント生成）を正式宣言するマイルストーンバージョン。
`cargo clean` による成果物クリーンアップを行い、フルビルド・全テスト通過を確認する。

**宣言文**:

> 「ステップ実行デバッガが、AI パイプラインの内部を露わにする。
>  時間を遡って本番障害を再現し、DAG 可視化が依存関係を一目で示す。
>  AI アドバイザーがプロファイリングデータを読み、次の最適化を提案する。
>
>  これが Favnir v68.0 — Developer Intelligence の姿である。」

ロードマップ `roadmap-v67.1-v68.0.md` の v68.0.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3515 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（本バージョンで `"68.0.0"` に更新する）
- `driver.rs` に `v67900_tests` が存在することを確認（`v68000_tests` の挿入位置）
- `driver.rs` に `v68000_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v67900_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `dev_intelligence_all_stable`, `debug_viz_suggest_docs_complete`
- `versions/current.md` の「進行中バージョン」が `v67.9.0` であることを確認

---

## 実装スコープ

### 1. `fav/Cargo.toml` — バージョン更新

```toml
version = "68.0.0"
```

（`"67.0.0"` → `"68.0.0"` に変更）

### 2. `MILESTONE.md` — v68.0.0 エントリを先頭に追加

既存の `## v67.0.0` エントリの直前に挿入:

```markdown
## v68.0.0（2026-08-06）— Developer Intelligence

> 「ステップ実行デバッガが、AI パイプラインの内部を露わにする。
>  時間を遡って本番障害を再現し、DAG 可視化が依存関係を一目で示す。
>  AI アドバイザーがプロファイリングデータを読み、次の最適化を提案する。
>
>  これが Favnir v68.0 — Developer Intelligence の姿である。」

**Developer Intelligence** の宣言バージョン。v67.1〜v67.9 で実装した
デバッグ・可視化・AI 提案・テストツール群の統合を宣言した。

**v67.1〜v67.9 達成内容:**
- v67.1（`fav debug`）: ステップ実行デバッガ（inspect / breakpoint / diff）
- v67.2（Time-Travel Debugging）: --record / --replay / rewind / forward
- v67.3（`fav viz`）: パイプライン DAG 可視化（ascii / svg / mermaid）
- v67.4（`fav suggest`）: AI 最適化アドバイザー（--from-profile）
- v67.5（`fav simulate`）: 合成データパイプラインテスト（--seed）
- v67.6（`Rune.proptest`）: Pipeline Property Testing（forall / shrink / --proptest-runs）
- v67.7（`fav profile --interactive`）: インタラクティブプロファイリング（drill / Suggestion）
- v67.8（`fav doc --math`）: 数式対応ドキュメント生成（MathJax / $$...$$）
- v67.9（安定化）: developer-intelligence.mdx / コードフリーズ

**テスト数**: 3519

---
```

### 3. `README.md` — v68.0.0 宣言を追加

既存の v67.0.0 / AI-Native Stage Layer の記述の直前に v68.0.0 の言及を追加。
`"Developer Intelligence"` または `"v68.0"` を含む必要がある（`readme_mentions_dev_intelligence` テストで検証）。

### 4. `CHANGELOG.md` — v68.0.0 エントリを先頭に追加

v67.1〜v67.9 で CHANGELOG 更新を保留していたため、v68.0.0 エントリに一括追記する。
既存の `## [v67.0.0]` エントリの直前に挿入:

```markdown
## [v68.0.0] — 2026-08-06 — Developer Intelligence 宣言 ★クリーンアップ

### Added
- `MILESTONE.md` に v68.0.0「Developer Intelligence」宣言文エントリを追加
- `v68000_tests`: 4 件追加（3515 → 3519 tests）
  - `cargo_toml_version_is_68_0_0`
  - `changelog_has_v68_0_0`
  - `milestone_has_dev_intelligence`
  - `readme_mentions_dev_intelligence`
- `site/content/docs/tools/developer-intelligence.mdx` 新規作成（v67.9.0）
- Developer Intelligence ツール群（v67.1〜v67.9）の成果を統合:
  - `fav debug`（v67.1）: ステップ実行デバッガ
  - Time-Travel Debugging（v67.2）: --record / --replay
  - `fav viz`（v67.3）: パイプライン DAG 可視化
  - `fav suggest --from-profile`（v67.4）: AI 最適化アドバイザー
  - `fav simulate`（v67.5）: 合成データパイプラインテスト
  - `Rune.proptest`（v67.6）: Pipeline Property Testing
  - `fav profile --interactive`（v67.7）: インタラクティブプロファイリング
  - `fav doc --math`（v67.8）: 数式対応ドキュメント生成
  - 安定化・コードフリーズ（v67.9）

### Changed
- `fav/Cargo.toml` version `"67.0.0"` → `"68.0.0"`
- `README.md` に Developer Intelligence 宣言を追記

### Note
- ★クリーンアップ（`cargo clean`）完了
- `cargo clean` 後は `fav/tmp/hello.fav` を復元すること（bootstrap テスト要件）

---
```

### 5. `driver.rs` — `v68000_tests` 追加

挿入位置: `// -- v67900_tests (v67.9.0)` コメントの直前

```rust
// -- v68000_tests (v68.0.0) -- Developer Intelligence 宣言 --
#[cfg(test)]
mod v68000_tests {
    #[test]
    fn cargo_toml_version_is_68_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(
            toml.contains("version = \"68.0.0\""),
            "Cargo.toml should have version 68.0.0: {}",
            &toml[..200.min(toml.len())]
        );
    }

    #[test]
    fn changelog_has_v68_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("v68.0.0"), "CHANGELOG.md should mention v68.0.0");
    }

    #[test]
    fn milestone_has_dev_intelligence() {
        let ms = include_str!("../../MILESTONE.md");
        assert!(
            ms.contains("Developer Intelligence"),
            "MILESTONE.md should contain 'Developer Intelligence'"
        );
    }

    #[test]
    fn readme_mentions_dev_intelligence() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Developer Intelligence") || readme.contains("v68.0"),
            "README.md should mention Developer Intelligence or v68.0"
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

- `fav/Cargo.toml` に `version = "68.0.0"` が含まれる
- `CHANGELOG.md` に `"v68.0.0"` が含まれる
- `MILESTONE.md` に `"Developer Intelligence"` が含まれる
- `README.md` に `"Developer Intelligence"` または `"v68.0"` が含まれる
- `cargo build` でエラーなし
- `cargo test --bin fav v68000_tests` で 4 件 PASS
- `cargo clean` 実行済み
- `fav/tmp/hello.fav` 復元済み
- `cargo test -j 8 -- --test-threads=8` で 3519 tests passed, 0 failed

---

## 非スコープ

- v69.x 以降のスプリント計画 — 別途策定

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../Cargo.toml"` → `fav/Cargo.toml`
- `"../../CHANGELOG.md"` → `favnir/CHANGELOG.md`
- `"../../MILESTONE.md"` → `favnir/MILESTONE.md`
- `"../../README.md"` → `favnir/README.md`

### `cargo clean` 後の `hello.fav` 復元理由

過去実績として `cargo clean` 後に `fav/tmp/hello.fav` が消えるケースが報告されているため、念のため必ず復元する。
`bootstrap_c2_artifact_roundtrip` テストが `fav/tmp/hello.fav` を参照するため、存在しないと FAIL する。

### テスト数の変化（+4）

マイルストーン宣言バージョン（x.0.0 リリース）では +4 テスト（サブバージョンの +2 とは異なる）。

### `readme_mentions_dev_intelligence` の OR 条件

`contains("Developer Intelligence") || contains("v68.0")` を使用。
README への記述は柔軟に対応できるよう OR 条件とする。
