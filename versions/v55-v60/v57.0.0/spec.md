# Spec — v57.0.0 — Language Power 2.0 宣言 ★クリーンアップ

## 概要

Language Power 2.0 スプリント（v56.1〜v56.9）の完成を正式に宣言するマイルストーンバージョン。
新機能追加なし。宣言エントリの追加・ドキュメント更新・cargo clean が主な成果物。

**宣言文**:

> 「ジェネリクスは制約で安全に縛られ、レコードは行変数で柔軟に合成され、
>  エフェクトは推論によって自然に表れる。
>  パターンはガード節と OR 構文で表現力を増し、モジュールは名前空間で整理される。
>  Favnir の型システムは開発者の意図を正確に表現できる。
>
>  これが Favnir v57.0 — Language Power 2.0 の姿である。」

---

## ロードマップ参照

- `versions/roadmap/roadmap-v56.1-v57.0.md` — v57.0.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v57.0.0 行
- ベーステスト数: **3248**（v56.9.0 完了時点の実績値）
- 目標テスト数: **3252**（+4）、かつ `cargo test` failures=0 かつテスト数 ≥ **3250**

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "57.0.0"
```

---

### 2. `MILESTONE.md` — Language Power 2.0 エントリ追加

ファイル先頭（`## v56.0.0` エントリの前）に追記する。

```markdown
## v57.0.0（2026-07-26）— Language Power 2.0

> 「ジェネリクスは制約で安全に縛られ、レコードは行変数で柔軟に合成され、
>  エフェクトは推論によって自然に表れる。
>  パターンはガード節と OR 構文で表現力を増し、モジュールは名前空間で整理される。
>  Favnir の型システムは開発者の意図を正確に表現できる。
>
>  これが Favnir v57.0 — Language Power 2.0 の姿である。」

**Language Power 2.0** の宣言バージョン。v56.1〜v56.9 の全機能統合を経て、
境界付きジェネリクス・行多相レコード・エフェクト推論 LSP・OR パターン・
as-パターン・モジュール名前空間の成熟を宣言する。

**v56.1〜v56.9 達成内容:**
- v56.1（境界付きジェネリクス本番品質化）: `where T: Interface` 正式化・E0422
- v56.2（複数 constraint・coherence 強化）: `T with Ord with Serialize`・E0423
- v56.3（行多相レコード活用拡張）: `{ field: Type | r }` 行変数明示・LSP ホバー
- v56.4（エフェクト推論 LSP 統合）: inlay hints・`fav check --show-types`
- v56.5（OR パターン + パターンガード強化）: `Ok(x) | Err(x)`・W037
- v56.6（パターンエイリアス）: `head @ { id, amount }` as-パターン
- v56.7（モジュール名前空間）: `import "path" as alias.*`・W038
- v56.8（ドキュメント）: bounded-generics / row-polymorphism / effect-inference MDX
- v56.9（安定化）: language-power2-overview.mdx 骨子・コードフリーズ
```

---

### 3. `README.md` — Language Power 2.0 宣言の追記

`v56.0` 宣言エントリの後に追記する。

```markdown
**v57.0（2026-07-26）で、[Language Power 2.0](./MILESTONE.md) マイルストーンを宣言しました。**
`where T: Interface` 本番品質化・行変数 `{ field: Type | r }` 明示・エフェクト推論 inlay hints・OR パターン・as-パターン・モジュール名前空間（`import "path" as alias.*`）が揃い、Favnir の型システムで開発者の意図を正確に表現できる状態になりました。
```

---

### 4. `fav/src/driver.rs` — `v57000_tests` 追加

`v56900_tests` の直前に挿入する。

**テスト 1: `cargo_toml_version_is_57_0_0`**

```rust
#[test]
fn cargo_toml_version_is_57_0_0() {
    let cargo_toml = include_str!("../Cargo.toml");
    assert!(
        cargo_toml.contains("version = \"57.0.0\""),
        "Cargo.toml version should be 57.0.0, got: {}",
        cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
    );
}
```

**テスト 2: `changelog_has_v57_0_0`**

```rust
#[test]
fn changelog_has_v57_0_0() {
    let changelog = include_str!("../../CHANGELOG.md");
    assert!(
        changelog.contains("[v57.0.0]"),
        "CHANGELOG.md should have a [v57.0.0] entry"
    );
}
```

**テスト 3: `milestone_has_language_power2`**

```rust
#[test]
fn milestone_has_language_power2() {
    let milestone = include_str!("../../MILESTONE.md");
    assert!(
        milestone.contains("Language Power 2.0"),
        "MILESTONE.md should declare Language Power 2.0"
    );
}
```

**テスト 4: `readme_mentions_language_power2`**

```rust
#[test]
fn readme_mentions_language_power2() {
    let readme = include_str!("../../README.md");
    assert!(
        readme.contains("Language Power 2.0"),
        "README.md should mention Language Power 2.0"
    );
}
```

---

### 5. `fav/src/driver.rs` — バージョンチェックテスト更新

`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.9.0"` → `"57.0.0"` に更新。
（モジュール名・関数名は慣例として変更しない）

---

### 6. ★クリーンアップ（`cargo clean`）

`cargo test` 全通過を確認した後、`cargo clean` を実行する。
次スプリント（v57.1〜v58.0）はクリーンなビルド状態から開始する。

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `cargo_toml_version_is_57_0_0` | `Cargo.toml` version が `"57.0.0"` である |
| `changelog_has_v57_0_0` | `CHANGELOG.md` が `"[v57.0.0]"` を含む |
| `milestone_has_language_power2` | `MILESTONE.md` が `"Language Power 2.0"` を含む |
| `readme_mentions_language_power2` | `README.md` が `"Language Power 2.0"` を含む |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3252 tests passed, 0 failed**、ベース 3248 + 4）
  （ロードマップの floor 条件は ≥ 3250）
- `cargo clippy -- -D warnings` クリーン
- `v57000_tests` 4件 全 pass
- `MILESTONE.md` に `"Language Power 2.0"` 宣言文エントリが追加されている
- `README.md` に `"Language Power 2.0"` の言及が追加されている
- `CHANGELOG.md` に `[v57.0.0]` エントリが追加されている
- `versions/current.md` が v57.0.0 / 3252 tests を反映
- 両ロードマップの v57.0.0 実績を COMPLETE に更新
- `cargo clean` 完了（★クリーンアップ）

---

## 備考

- **`include_str!` のパス**:
  - `CHANGELOG.md`: `"../../CHANGELOG.md"` — `fav/src/` → `fav/` → プロジェクトルート
  - `MILESTONE.md`: `"../../MILESTONE.md"` — 同上
  - `README.md`: `"../../README.md"` — 同上
- **`cargo clean` のタイミング**: `cargo test` 全通過後・ポスト処理完了後に実行。
  クリーン後は次回 `cargo build` で全ファイルが再コンパイルされることを確認する。
- **テスト数**: `v57000_tests` に 4 件追加。ベース 3248 + 4 = 3252。
- **`v56300_tests` 慣例**: `cargo_toml_version_is_56_3_0` 関数名はそのまま維持し、期待値のみ更新する。
