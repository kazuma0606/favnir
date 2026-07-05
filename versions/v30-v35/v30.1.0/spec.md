# v30.1.0 Spec — ビルド軽量化

**バージョン**: 30.1.0
**日付**: 2026-07-01
**フェーズ**: Real-World Readiness（phase 1/9）
**前バージョン**: v30.0.0（Ecosystem Maturity 宣言）

---

## 概要

スプリントを重ねるごとに `fav/target/debug/` が 40GB+ に膨らむ問題を解消する。

`[profile.dev] debug = 0` でデバッグシンボルを無効化し、
`split-debuginfo = "off"` でデバッグ情報分割も抑制することで、
`deps/`（現在 20GB）を中心にビルド生成物を削減する。

この変更は Cargo の標準設定であり、テスト・実行・CI への影響はない。

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `fav/Cargo.toml` | `[profile.dev]` セクション追加（`debug = 0` / `split-debuginfo = "off"`）|
| `fav/Cargo.toml` | version `30.0.0` → `30.1.0` |
| `fav/Cargo.lock` | `cargo build` 実行後に変更があればコミットに含める |
| `CHANGELOG.md` | `[v30.1.0]` セクション追加 |
| `benchmarks/v30.1.0.json` | ベンチマーク記録（test_count: 2378）|
| `fav/src/driver.rs` | `v301000_tests` 6 件追加 |
| `versions/current.md` | 進行中バージョンを `v30.1.0` に更新 |
| `versions/v30-v35/v30.1.0/tasks.md` | 実装完了後 COMPLETE に更新 |

---

## 実装内容

### Cargo.toml — `[profile.dev]` セクション追加

```toml
# fav/Cargo.toml 末尾に追加
[profile.dev]
debug = 0
split-debuginfo = "off"
```

| 設定 | 効果 |
|---|---|
| `debug = 0` | デバッグシンボル（`.pdb` / DWARF）を生成しない。`deps/` のサイズが 30〜40% 削減される |
| `split-debuginfo = "off"` | macOS の dSYM / Linux の `.dwp` 分割ファイルを無効化（Windows では `debug = 0` が主担当）|

### 影響範囲

- `cargo build` の出力バイナリ（`fav.exe` / `rvm.exe`）のサイズ変化なし
- `cargo test` の動作変化なし（デバッグシンボルは Rust テストの実行に不要）
- `cargo clippy` の動作変化なし
- CI（GitHub Actions）への影響なし（CI は `release` プロファイルを使用しないため）
- Windows 環境（`.pdb` ファイルが生成されなくなる）が主な恩恵

### スコープ外

- `fav/.cargo/config.toml` による LLD リンカー設定（効果が不確実なため今回は見送り）
- `[profile.release]` の最適化設定（本番ビルドの変更は別バージョンで実施）
- `duckdb` / `wasmtime` の optional feature 化（大規模リファクタのため別バージョン）

---

## テスト戦略

### v301000_tests（6 件）

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_30_1_0` | `fav/Cargo.toml` が `30.1.0` を含む |
| `cargo_toml_has_profile_dev` | `fav/Cargo.toml` が `[profile.dev]` を含む |
| `profile_dev_debug_is_zero` | `fav/Cargo.toml` が `debug = 0` を含む |
| `profile_dev_split_debuginfo_off` | `fav/Cargo.toml` が `split-debuginfo = "off"` を含む |
| `changelog_has_v30_1_0` | `CHANGELOG.md` に `[v30.1.0]` が存在する |
| `benchmark_v30_1_0_exists` | `benchmarks/v30.1.0.json` が `"30.1.0"` を含む |

テスト数: 2372 → **2378**（+6）

---

## 完了条件

- [ ] `Cargo.toml` version = "30.1.0"
- [ ] `Cargo.toml` に `[profile.dev]` セクションが存在する
- [ ] `[profile.dev]` に `debug = 0` が設定されている
- [ ] `[profile.dev]` に `split-debuginfo = "off"` が設定されている
- [ ] `cargo build` がエラーなく完了する
- [ ] `cargo test` — 2378 tests PASS
- [ ] `CHANGELOG.md` に `[v30.1.0]` セクションあり
- [ ] `benchmarks/v30.1.0.json` 存在（test_count: 2378）
- [ ] `cargo test --bin fav v301000` — 6/6 PASS
- [ ] tasks.md を COMPLETE に更新

---

## 検証コマンド

```bash
cd /c/Users/yoshi/favnir/fav

# ビルド確認
cargo build 2>&1 | tail -3

# テスト確認
cargo test --bin fav v301000 2>&1 | tail -5
cargo test 2>&1 | grep "test result"

# ビルドサイズ確認（debug=0 適用前後の比較）
du -sh target/
```
