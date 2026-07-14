# v41.2.0 実装計画

## 実装順序

1. **error_catalog.rs** — E0404 / E0405 / E0406 追加（E0403 直後・ErrorEntry 形式）
2. **checker.fav** — TypeDef `invariants: List<String>` フィールド追加 + `check_refinement_alias` 統合呼び出し
3. **Cargo.toml** — version bump `41.1.0` → `41.2.0`
4. **CHANGELOG.md** — `[v41.2.0]` エントリ追加
5. **driver.rs** — `v41100_tests::cargo_toml_version_is_41_1_0` スタブ化 + `v41200_tests` 追加
6. **cargo test** — 2851 tests passed 確認
7. **versions/current.md** 更新 + roadmap マーク

**checker.rs 変更不要**: `type_invariants: HashMap<String, Vec<Expr>>` が既存フィールド（line 912）として存在し、TypeDef 処理（line 2158-2159）で `td.invariants.clone()` を既に収集している。v41.1.0 の parser 変更により Alias 型の invariants も自動収集済み。

## 各ステップ詳細

### Step 1: error_catalog.rs

- `E0403` の `fix` フィールド終わり直後（`},` の次行）に `// ── E04xx: Refinement type (v41.2.0)` コメント付きで追加
- E0404/E0405/E0406 の 3 エントリを `ErrorEntry { code, title, category, description, example, fix }` 形式で追加

### Step 2: checker.fav

- T0 で実際の TypeDef フィールドリストと型定義チェック関数名を確認
- `TypeDef` レコード型に `invariants: List<String>` を末尾フィールドとして追加
- `check_refinement_alias` stub（v41.1.0 追加済み）を型定義チェック関数内から呼び出す

### Step 3-4: バージョン管理ファイル

標準手順に従い bump + CHANGELOG エントリ。

### Step 5: driver.rs

- `v41100_tests` の `cargo_toml_version_is_41_1_0` をスタブ化
- `v41200_tests` モジュールを末尾に追加（3 テスト、`use super::*` 不要）

## パス確認

| `include_str!` | 解決先 |
|---|---|
| `include_str!("../Cargo.toml")` | `fav/Cargo.toml` |
| `include_str!("../../CHANGELOG.md")` | `CHANGELOG.md`（ルート） |
| `include_str!("error_catalog.rs")` | `fav/src/error_catalog.rs`（同ディレクトリ） |
