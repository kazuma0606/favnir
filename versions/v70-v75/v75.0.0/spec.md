# v75.0.0 仕様書 — Favnir 2.0 宣言 ★クリーンアップ

Date: 2026-08-14

---

## Background

v74.1〜v74.9 で完了した全スプリントを踏まえ、Favnir 2.0 を正式宣言するマイルストーンバージョン。
新規機能の追加は行わず、バージョン更新・MILESTONE.md / README.md 更新・`cargo clean` クリーンアップ・
宣言テスト 4 件の追加のみを行う。

宣言文（ロードマップより）:
> 「compiler.fav が Favnir を完全に記述し、型システムが次元と制約を保証する。
>  依存型がベクトルの次元を守り、refined type がゼロ除算をコンパイル時に止める。
>  VS Code がパイプラインを補完し、AI がエラーを修正し、
>  実際のデータチームが本番で Favnir を走らせている。
>
>  データコントラクトがスキーマ境界を守り、品質スコアが劣化を警告する。
>  Favnir が Favnir 自身を運用し、Rune マーケットプレイスが
>  コミュニティの知恵を型安全なピースとして流通させる。
>
>  これが Favnir v75.0 — Favnir 2.0 の姿である。」

---

## Goals

1. `v75000_tests` モジュール（4 件）を `driver.rs` に追加する
   - `cargo_toml_version_is_75_0_0` — Cargo.toml が `version = "75.0.0"` を持つことを確認
   - `changelog_has_v75_0_0` — CHANGELOG.md に `[v75.0.0]` エントリが存在することを確認
   - `milestone_has_favnir_2` — MILESTONE.md に「Favnir 2.0」が記載されることを確認
   - `readme_mentions_favnir_2` — README.md に `v75.0` または「Favnir 2.0」が記載されることを確認
2. `cargo clean` で build artifacts をクリーンアップする
3. バージョンを `74.9.0` → `75.0.0` に更新する
4. MILESTONE.md / README.md に Favnir 2.0 宣言を追記する

---

## テスト仕様

### `cargo_toml_version_is_75_0_0`

```rust
let cargo_toml = include_str!("../Cargo.toml");
assert!(cargo_toml.contains("version = \"75.0.0\""), "Cargo.toml version should be 75.0.0");
```

### `changelog_has_v75_0_0`

```rust
let changelog = include_str!("../../CHANGELOG.md");
assert!(changelog.contains("[v75.0.0]"), "CHANGELOG.md should have v75.0.0 entry");
```

### `milestone_has_favnir_2`

```rust
let milestone = include_str!("../../MILESTONE.md");
assert!(milestone.contains("Favnir 2.0"), "MILESTONE.md should mention Favnir 2.0");
```

### `readme_mentions_favnir_2`

```rust
let readme = include_str!("../../README.md");
assert!(readme.contains("v75.0") || readme.contains("Favnir 2.0"),
    "README.md should mention v75.0 or Favnir 2.0");
```

**注意:** `cargo_toml_version_is_75_0_0` は過去の宣言テスト（v73000_tests 等）と同様に、
次バージョンリリース時の `replace_all` で最新バージョン値に更新される設計。

---

## Success Criteria

1. `v75000_tests` 4 件全て pass する
2. `cargo test` 全体で 3692 tests pass（0 failures）
3. MILESTONE.md に「Favnir 2.0」が記載されている
4. README.md に「Favnir 2.0」または「v75.0」が記載されている

---

## スコープ外（明示的除外）

- 新規機能・新規構造体・新規関数の追加
- CI パイプラインの変更
- 次フェーズ（v75.1.0〜）のロードマップ策定（宣言後に別途実施）
- `site/` MDX 追加

---

## Error Codes

新規エラーコードなし

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v75000_tests` 4 件追加（v749000_tests の直後） |
| `fav/Cargo.toml` | `version = "75.0.0"` に更新 |
| `CHANGELOG.md` | v75.0.0 エントリを先頭に追加 |
| `MILESTONE.md` | 「Favnir 2.0」宣言を追記 |
| `README.md` | v75.0 達成（Favnir 2.0）を追記 |
| `versions/current.md` | 完了バージョン・次フェーズを更新 |
| `versions/roadmap/roadmap-v74.1-v75.0.md` | スプリント状態を「完了」に更新 |

**クリーンアップ:**
- `cargo clean` 実施（target/ 削除）
- `cargo clean` 後、`fav/tmp/hello.fav` が消えている場合は復元する
  （内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`）
