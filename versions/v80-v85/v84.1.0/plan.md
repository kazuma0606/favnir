# plan: v84.1.0 — E2E ショーケース基盤

## 実装ステップ（依存順）

### Step 1: 事前確認

- `cargo test` を実行し、3,909 tests, 0 failures を確認する
- `Cargo.toml` バージョンが `84.0.0` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する）
- `fav/src/driver.rs` に `mod v84000_tests` が存在することを確認する

### Step 2: infra/e2e-demo/favnir4-showcase/ 作成

ディレクトリを作成し、4 ファイルを配置する。

#### Step 2-1: pipeline.fav

4 Quality 柱を統合するパイプラインの骨格を Favnir 構文で記述する。
- `load_stage` / `transform_stage` / `quality_stage` / `observe_stage` の 4 関数
- 各関数は `Result<List<Row>, String>` を返すプレースホルダ実装
- `AppCtx` 型を使用（v13.x で導入済み）

#### Step 2-2: fav.toml

`[package]` / `[quality]` / `[contract]` / `[observe]` の 4 セクションを含む設定ファイル。
- `[quality]` : gate モード（permissive）と rules の例
- `[contract]` : input contract 名と SLA 目標（ms）
- `[observe]` : アラート閾値と SLO 目標（%）

#### Step 2-3: contract.fav

`Favnir4ShowcaseContract` 型を宣言する。
- `input_fields: List<String>`
- `output_fields: List<String>`
- `sla_ms: Int`

#### Step 2-4: README.md

ショーケースの概要・前提・実行手順を記述する。
```
# Favnir 4.0 Showcase
## 概要
## 前提
## 実行方法
```

### Step 3: driver.rs に v84100_tests を追加

`mod v84000_tests` の直後に `#[cfg(test)] mod v84100_tests` を追加する。
`use super::*` は不要（外部シンボル不使用）。

#### Test 1: favnir4_showcase_structure_exists

```rust
#[test]
fn favnir4_showcase_structure_exists() {
    // 4 ファイルが存在することを確認
    assert!(std::path::Path::new("../infra/e2e-demo/favnir4-showcase/pipeline.fav").exists());
    assert!(std::path::Path::new("../infra/e2e-demo/favnir4-showcase/fav.toml").exists());
    assert!(std::path::Path::new("../infra/e2e-demo/favnir4-showcase/contract.fav").exists());
    assert!(std::path::Path::new("../infra/e2e-demo/favnir4-showcase/README.md").exists());
}
```

#### Test 2: favnir4_showcase_contract_valid

```rust
#[test]
fn favnir4_showcase_contract_valid() {
    // contract.fav に Favnir4ShowcaseContract が含まれることを確認
    let content = include_str!("../../infra/e2e-demo/favnir4-showcase/contract.fav");
    assert!(content.contains("Favnir4ShowcaseContract"));
    // fav.toml に [quality], [contract], [observe] が含まれることを確認
    let toml = include_str!("../../infra/e2e-demo/favnir4-showcase/fav.toml");
    assert!(toml.contains("[quality]"));
    assert!(toml.contains("[contract]"));
    assert!(toml.contains("[observe]"));
}
```

### Step 4: cargo test で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、3,911 tests, 0 failures を確認する。

### Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v84.1.0 エントリを追加する。

> 注意: `v84100_tests` には `changelog_has_v84_1_0` テストが含まれないため、
> CHANGELOG 更新は Step 4 の後でよい。

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
