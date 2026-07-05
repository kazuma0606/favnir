# v34.9.0 — Spec

## 概要

**テーマ**: `fav upgrade` ドキュメント完全化 + ctx 移行テストフィクスチャ拡充

**方針**: v34.8.0 で実装した `fav upgrade` コマンドの公式ドキュメントを
`site/content/docs/tools/upgrade-guide.mdx` として新規作成する。
あわせて `fav/tests/fixtures/ctx_migration/` に Before/After フィクスチャを追加し、
ctx 移行シリーズ（v34.5〜v34.9）の品質基盤を固める。

---

## 背景

v34.8.0 で `MIGRATION.md` と `cmd_upgrade` を実装したが、
`site/content/docs/tools/` 配下の公式ドキュメントが未整備のまま。
v35.0 の Production Ready 条件「ドキュメントを読めば新しいエンジニアが 1 日で使い始められる」
を満たすためには upgrade-guide.mdx が必要。
また、ctx 移行の Before/After フィクスチャを `fav/tests/fixtures/` に追加することで
将来の regression test の基盤を整える。

### 既存実装の確認

| 機能 | 状態 | 備考 |
|---|---|---|
| `fav upgrade --from-effects` | 実装済み（v34.8.0） | driver.rs `cmd_upgrade` |
| `MIGRATION.md` | 実装済み（v34.8.0） | repo root |
| `site/content/docs/tools/upgrade-guide.mdx` | **未実装 → 本バージョンで新規作成** | |
| `fav/tests/fixtures/ctx_migration/` | **未実装 → 本バージョンで新規作成** | |

### ロードマップからの設計判断

| 項目 | ロードマップ定義 | 本 spec の判断 |
|---|---|---|
| テストカバレッジの向上 | v34.8〜v34.9 | **フィクスチャ追加で対応** |
| パフォーマンスチューニング | v34.8〜v34.9 | **v35.0 での cargo clean 時に対応** |
| CHANGELOG / MIGRATION ガイド整備 | v34.8〜v34.9 | **v34.8 で完了済み** |

---

## 実装スコープ

### 新規ファイル

```
site/content/docs/tools/upgrade-guide.mdx      fav upgrade コマンド公式ドキュメント
fav/tests/fixtures/ctx_migration/before.fav    ctx 移行前フィクスチャ（!Http 使用）
fav/tests/fixtures/ctx_migration/after.fav     ctx 移行後フィクスチャ（AppCtx 使用）
```

### 変更ファイル

1. `fav/Cargo.toml` — version `34.8.0` → `34.9.0`
2. `fav/src/driver.rs` — `cargo_toml_version_is_34_8_0` スタブ化 + `v349000_tests` 5件追加
3. `CHANGELOG.md` — `[v34.9.0]` セクション先頭追記
4. `benchmarks/v34.9.0.json` — 新規作成
5. `versions/current.md` — 最新安定版を v34.9.0 に更新

---

## upgrade-guide.mdx 仕様

タイトル: `fav upgrade — プロジェクトアップグレードガイド`

含むべき内容:
- `fav upgrade --from-effects` の概要と使い方
- `--dry-run` / `--in-place` フラグの説明
- `fav migrate --from-effects`（単一ファイル）との使い分け
- ステップバイステップのワークフロー
- トラブルシューティング

**含むべきキーワード**: `"fav upgrade"` / `"--from-effects"`（アサーション対象）

---

## フィクスチャ仕様

### `fav/tests/fixtures/ctx_migration/before.fav`

```favnir
// ctx 移行前: !Http エフェクトを使用（W022 警告が発生する）
fn fetch_orders(url: String) -> Result<String, String> !Http {
    HTTP.get(url)
}
```

含むべきキーワード: `"!Http"`（アサーション対象）

### `fav/tests/fixtures/ctx_migration/after.fav`

```favnir
// ctx 移行後: AppCtx パラメータを使用
import runes/ctx

fn fetch_orders(ctx: AppCtx, url: String) -> Result<String, String> {
    bind { http } <- ctx
    http.get(url)
}
```

含むべきキーワード: `"AppCtx"`（アサーション対象）

---

## テスト仕様（v349000_tests）

```rust
// ── v34.9.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v349000_tests {
    #[test]
    fn cargo_toml_version_is_34_9_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.9.0"), "Cargo.toml must contain '34.9.0'");
    }

    #[test]
    fn upgrade_guide_exists() {
        let src = include_str!("../../site/content/docs/tools/upgrade-guide.mdx");
        assert!(
            src.contains("fav upgrade"),
            "upgrade-guide.mdx must document fav upgrade command"
        );
    }

    #[test]
    fn upgrade_guide_covers_from_effects() {
        let src = include_str!("../../site/content/docs/tools/upgrade-guide.mdx");
        assert!(
            src.contains("--from-effects"),
            "upgrade-guide.mdx must document --from-effects flag"
        );
    }

    #[test]
    fn ctx_migration_before_fixture_exists() {
        let src = include_str!("../tests/fixtures/ctx_migration/before.fav");
        assert!(
            src.contains("!Http"),
            "before.fav fixture must contain !Http effect"
        );
    }

    #[test]
    fn ctx_migration_after_fixture_exists() {
        let src = include_str!("../tests/fixtures/ctx_migration/after.fav");
        assert!(
            src.contains("AppCtx"),
            "after.fav fixture must contain AppCtx"
        );
    }
}
```

### 設計注記

- `use super::*` は**不要**（`include_str!` のみ使用）
- WASM ゲートなし
- `upgrade-guide.mdx` パス: `../../site/content/docs/tools/upgrade-guide.mdx`
  （`fav/src/` → `../../` = `favnir/`）
- フィクスチャパス: `../tests/fixtures/ctx_migration/before.fav`
  （`fav/src/` → `../` = `fav/`）
- v349000_tests は v348000_tests 直後・`// ── v31.7.0 tests` の前に挿入

---

## 完了条件

- [ ] `cargo clean` 不要（x.9.0 のため実施しない）
- [ ] `Cargo.toml` version = `"34.9.0"`
- [ ] `cargo_toml_version_is_34_8_0` が空スタブになっていること
- [ ] `cargo test --bin fav v349000` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（2581 件想定 = 2576 + 5、0 failures）
- [ ] `site/content/docs/tools/upgrade-guide.mdx` が存在し `"fav upgrade"` と `"--from-effects"` を含むこと
- [ ] `fav/tests/fixtures/ctx_migration/before.fav` が存在し `"!Http"` を含むこと
- [ ] `fav/tests/fixtures/ctx_migration/after.fav` が存在し `"AppCtx"` を含むこと
- [ ] `CHANGELOG.md` に `[v34.9.0]` セクション
- [ ] `benchmarks/v34.9.0.json` 存在かつ `tests_passed` が実測値
- [ ] `versions/current.md` が v34.9.0 に更新されていること
- [ ] `tasks.md` が COMPLETE
