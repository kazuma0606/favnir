# v34.8.0 — Spec

## 概要

**テーマ**: MIGRATION ガイド整備 + `fav upgrade` コマンド

**方針**: v34.5〜v34.7 で完成した !Effect → Capability Context 移行シリーズの締めくくりとして、
ユーザー向け移行ガイド（`MIGRATION.md`）を新規作成し、
プロジェクト一括アップグレードコマンド `fav upgrade` を実装する。

---

## 背景

v34.5.0 で W022 lint + IoCtx、v34.6.0 で DbCtx/HttpClient/StreamClient、
v34.7.0 でドキュメントを整備した。
v34.8.0 はこのシリーズの実用化フェーズ：
外部ユーザーが既存プロジェクトを移行するための公式手順書と CLI ツールを提供する。

### 既存実装の確認

| 機能 | 状態 | 備考 |
|---|---|---|
| `fav migrate --from-effects` | 実装済み（v13.10.0） | driver.rs `cmd_migrate` |
| W022 lint ルール | 実装済み（v34.5.0） | lint.rs |
| IoCtx / DbCtx / HttpClient / StreamClient | 実装済み（v34.5〜6） | runes/ctx/ |
| `MIGRATION.md`（トップレベル） | **未実装 → 本バージョンで新規作成** | |
| `fav upgrade` コマンド | **未実装 → 本バージョンで実装** | |

### ロードマップからの設計判断

| 項目 | ロードマップ定義 | 本 spec の判断 |
|---|---|---|
| CHANGELOG / MIGRATION ガイド整備 | v34.8〜v34.9 | **本バージョンで実施** |
| `fav upgrade`（移行支援ツール） | v34.8〜v34.9 | **本バージョンで実施** |
| テストカバレッジ向上（3000+ 目標） | v34.8〜v34.9 | **v34.9 以降で対応** |

---

## 実装スコープ

### 新規ファイル

```
MIGRATION.md   !Effect → Capability Context 移行の完全ガイド
```

### 変更ファイル

1. `fav/Cargo.toml` — version `34.7.0` → `34.8.0`
2. `fav/src/driver.rs` — `pub fn cmd_upgrade` 実装 + `cargo_toml_version_is_34_7_0` スタブ化 + `v348000_tests` 5件追加
3. `fav/src/main.rs` — `Some("upgrade")` アーム追加
4. `CHANGELOG.md` — `[v34.8.0]` セクション先頭追記
5. `benchmarks/v34.8.0.json` — 新規作成
6. `versions/current.md` — 最新安定版を v34.8.0 に更新

---

## MIGRATION.md 仕様

タイトル: `Migration Guide — !Effect → Capability Context`

含むべき内容:
- 移行の背景（v34.5〜v34.7 系列）
- `fav upgrade --from-effects` を使った自動移行手順
- `!Effect` → ctx フィールド対応表
- 手動移行のステップバイステップ手順
- Before / After コード例
- FAQ（よくある質問）

**含むべきキーワード**: `"AppCtx"` / `"fav upgrade"`（アサーション対象）

---

## `fav upgrade` コマンド仕様

```
fav upgrade --from-effects [--dry-run | --in-place]
```

| フラグ | 動作 |
|---|---|
| `--from-effects --dry-run` | プロジェクト全体の移行内容をプレビュー表示（変更なし）|
| `--from-effects --in-place` | プロジェクト全体の `.fav` ファイルを一括移行 |
| `--from-effects`（フラグなし） | dry-run 旨のガイダンスを表示 |
| フラグなし | Err — 使い方を案内 |

### `cmd_upgrade` シグネチャ

```rust
pub fn cmd_upgrade(args: &[&str]) -> Result<String, String>
```

### main.rs ディスパッチ

```rust
Some("upgrade") => {
    let rest: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
    match cmd_upgrade(&rest) {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
```

---

## テスト仕様（v348000_tests）

```rust
// ── v34.8.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v348000_tests {
    use super::*;

    #[test]
    fn cargo_toml_version_is_34_8_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.8.0"), "Cargo.toml must contain '34.8.0'");
    }

    #[test]
    fn migration_guide_exists() {
        let src = include_str!("../../MIGRATION.md");
        assert!(
            src.contains("AppCtx"),
            "MIGRATION.md must document AppCtx migration"
        );
    }

    #[test]
    fn migration_guide_covers_upgrade_cmd() {
        let src = include_str!("../../MIGRATION.md");
        assert!(
            src.contains("fav upgrade"),
            "MIGRATION.md must document fav upgrade command"
        );
    }

    #[test]
    fn cmd_upgrade_returns_ok_for_dry_run() {
        let result = cmd_upgrade(&["--from-effects", "--dry-run"]);
        assert!(
            result.is_ok(),
            "cmd_upgrade --from-effects --dry-run must succeed: {:?}",
            result
        );
    }

    #[test]
    fn cmd_upgrade_requires_flag() {
        let result = cmd_upgrade(&[]);
        assert!(
            result.is_err(),
            "cmd_upgrade with no args must return Err"
        );
    }
}
```

### 設計注記

- `use super::*` が**必要**（`cmd_upgrade` 関数を直接呼ぶため）
- WASM ゲートなし
- `include_str!` パス: `../../MIGRATION.md`（`fav/src/` → `../../` = `favnir/`）
- v348000_tests は v347000_tests 直後・`// ── v31.7.0 tests` の前に挿入

---

## 完了条件

- [ ] `cargo clean` 不要（x.8.0 のため実施しない）
- [ ] `Cargo.toml` version = `"34.8.0"`
- [ ] `cargo_toml_version_is_34_7_0` が空スタブになっていること
- [ ] `cargo test --bin fav v348000` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（2576 件想定 = 2571 + 5、0 failures）
- [ ] `MIGRATION.md` が存在し `"AppCtx"` と `"fav upgrade"` を含むこと
- [ ] `cmd_upgrade(&["--from-effects", "--dry-run"])` が `Ok` を返すこと
- [ ] `cmd_upgrade(&[])` が `Err` を返すこと
- [ ] `CHANGELOG.md` に `[v34.8.0]` セクション
- [ ] `benchmarks/v34.8.0.json` 存在かつ `tests_passed` が実測値
- [ ] `versions/current.md` が v34.8.0 に更新されていること
- [ ] `tasks.md` が COMPLETE
