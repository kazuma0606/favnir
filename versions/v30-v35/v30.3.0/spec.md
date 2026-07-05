# v30.3.0 Spec — マルチファイルプロジェクト E2E 検証

**バージョン**: 30.3.0
**日付**: 2026-07-01
**フェーズ**: Real-World Readiness（phase 3/9）
**前バージョン**: v30.2.0（postgres-etl テンプレート v2）

---

## 概要

`fav.toml` プロジェクトモード（複数 .fav ファイル）が実用レベルで動作するか検証する。

検証対象は `fav/tests/fixtures/multifile_etl/` フィクスチャプロジェクト。
各コマンド（check / lint / fmt）で発見したバグを修正し、Rust テストとして固定する。

### 本バージョンのスコープ外

- `fav run`：`Pipeline` stage がデータ入力を必要とするため、入力形式の設計が未確定。v30.5 のドッグフード実装時に検証する。
- `fav test`：フィクスチャに `test {}` ブロックを持つファイルがないため v30.3 では対象外。v30.5 以降で検証する。

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `fav/Cargo.toml` | version `30.2.0` → `30.3.0` |
| `fav/Cargo.lock` | `cargo build` 実行後に変更があればコミットに含める |
| `fav/tests/fixtures/multifile_etl/` | マルチファイル検証用フィクスチャプロジェクト（新規作成）|
| `fav/src/driver.rs` | `v303000_tests` 7 件追加 |
| `CHANGELOG.md` | `[v30.3.0]` セクション追加 |
| `benchmarks/v30.3.0.json` | ベンチマーク記録（test_count: 2391）|
| `versions/current.md` | 進行中バージョンを `v30.3.0` に更新 |
| `versions/v30-v35/v30.3.0/tasks.md` | 実装完了後 COMPLETE に更新 |
| site/ MDX | 本バージョンはスコープ外（ロードマップに記載なし）|

---

## フィクスチャプロジェクト構成

```
fav/tests/fixtures/multifile_etl/
├── fav.toml
└── src/
    ├── types.fav
    ├── validators.fav
    └── main.fav
```

### `fav.toml`

```toml
[project]
name    = "multifile_etl"
version = "0.1.0"
edition = "2026"
src     = "src"
```

### `src/types.fav`

```favnir
type RawRow = {
    id:     String
    name:   String
    amount: String
}

type ValidRow = {
    id:     Int
    name:   String
    amount: Float
}

type RowError = {
    field:   String
    message: String
}
```

### `src/validators.fav`

`String.to_int` / `String.to_float` は `Option<T>` を返す（VM 確認済み）。
`Ok`/`Err` ではなく `Some`/`None` パターンを使用すること。

```favnir
import src/types

public fn validate_row(row: RawRow) -> Result<ValidRow, RowError> {
    match String.to_int(row.id) {
        None     => Result.err({ field: "id", message: "id must be integer" })
        Some(id) => match String.to_float(row.amount) {
            None         => Result.err({ field: "amount", message: "amount must be float" })
            Some(amount) => Result.ok({ id: id, name: row.name, amount: amount })
        }
    }
}
```

### `src/main.fav`

`Result.all(List.map(...))` は `Result<List<ValidRow>, RowError>` を返す。
stage の返値型をそのまま宣言すること。

```favnir
import src/types
import src/validators

public stage Pipeline: List<RawRow> -> Result<List<ValidRow>, RowError> = |rows| {
    Result.all(List.map(rows, |row| validators.validate_row(row)))
}
```

---

## VM プリミティブ確認

| 関数 | 返値型 | 確認 |
|---|---|---|
| `String.to_int` | `Option<Int>` | ✓ checker.rs 6196 |
| `String.to_float` | `Option<Float>` | ✓ checker.rs 6197 |
| `List.map` | `List<B>` | ✓ vm.rs 3235 |
| `Result.all` | `Result<List<T>, E>` | ✓ vm.rs 4434 |

---

## 検証コマンド（手動）

```bash
cd /c/Users/yoshi/favnir/fav

cargo build 2>&1 | tail -2

# 型チェック
./target/debug/fav check tests/fixtures/multifile_etl/src/types.fav
./target/debug/fav check tests/fixtures/multifile_etl/src/validators.fav
./target/debug/fav check tests/fixtures/multifile_etl/src/main.fav

# lint（警告なし）
./target/debug/fav lint tests/fixtures/multifile_etl/src/main.fav
./target/debug/fav lint tests/fixtures/multifile_etl/src/validators.fav

# fmt --check（フォーマット確認）
./target/debug/fav fmt --check tests/fixtures/multifile_etl/src/types.fav
./target/debug/fav fmt --check tests/fixtures/multifile_etl/src/validators.fav
./target/debug/fav fmt --check tests/fixtures/multifile_etl/src/main.fav
```

---

## テスト戦略

### v303000_tests（7 件）

`Parser::parse_str` を使い、フィクスチャファイルが構文エラーなく解析できることを確認する。
第 2 引数には固定文字列を渡す（OS によるパス区切り差異を回避）。

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_30_3_0` | `fav/Cargo.toml` が `30.3.0` を含む |
| `multifile_fixture_fav_toml_exists` | `fav.toml` が存在する |
| `multifile_fixture_types_fav_parses` | `src/types.fav` がパース成功 |
| `multifile_fixture_validators_fav_parses` | `src/validators.fav` がパース成功 |
| `multifile_fixture_main_fav_parses` | `src/main.fav` がパース成功 |
| `changelog_has_v30_3_0` | `CHANGELOG.md` に `[v30.3.0]` が存在する |
| `benchmark_v30_3_0_exists` | `benchmarks/v30.3.0.json` が `"30.3.0"` を含む |

テスト数: 2384 → **2391**（+7）

---

## 完了条件

- [ ] `Cargo.toml` version = "30.3.0"
- [ ] `fav/tests/fixtures/multifile_etl/` が 4 ファイルで構成されている（fav.toml + 3 .fav）
- [ ] `fav check` が各 .fav ファイルで通ること（手動検証）
- [ ] `fav lint` が各 .fav ファイルで通ること（手動検証）
- [ ] `fav fmt --check` が各 .fav ファイルで通ること（手動検証）
- [ ] `cargo test` — 2391 tests PASS
- [ ] `CHANGELOG.md` に `[v30.3.0]` セクションあり
- [ ] `benchmarks/v30.3.0.json` 存在（test_count: 2391）
- [ ] `cargo test --bin fav v303000` — 7/7 PASS
- [ ] `versions/current.md` を `v30.3.0` に更新
- [ ] tasks.md を COMPLETE に更新
