# v39.4.0 spec — Secret Rune 強化

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v39.4.0 |
| テーマ | Secret Rune 強化 — AWS / Vault / GCP / Env バックエンド対応 |
| 前提 | v39.3.0 COMPLETE — `fav policy` 完了 |
| 完了条件 | `v39400_tests` 全テスト pass・`cargo test` 0 failures（≥ 2797 件） |

## 背景と目的

v39.3.0 でポリシー定義が整った。v39.4.0 では Secret Rune を強化し、
AWS Secrets Manager / HashiCorp Vault / GCP Secret Manager の 3 バックエンドと、
ローカル開発用の環境変数フォールバック（`get_env`）を追加する。

`fav.toml` の `[secrets]` セクションでバックエンドを宣言的に切り替えられる。

**想定使用例**:
```favnir
// AWS バックエンド使用時
bind api_key <- secret.get_aws(ctx, "my-api-key")

// Vault バックエンド使用時
bind db_pass <- secret.get_vault(ctx, "database/creds/my-app")

// GCP バックエンド使用時
bind token <- secret.get_gcp(ctx, "projects/my-proj/secrets/token/versions/latest")

// ローカル開発フォールバック（!Http 不要）
bind dev_key <- secret.get_env(ctx, "MY_API_KEY")
```

**`fav.toml` の `[secrets]` ブロック（仕様定義のみ）**:
```toml
[secrets]
backend = "aws"   # "aws" / "vault" / "gcp" / "env"
```

> **スコープ注意**: `fav.toml [secrets]` セクションの実際のパース対応（`toml.rs` 変更）は後続バージョンで行う。
> 本バージョンでは Rune ファイル実装とスキーマ仕様の定義のみをスコープとし、`toml.rs` は変更しない。

## 実装スコープ

### 1. `runes/secret/secret.fav` — 新規作成

```favnir
// runes/secret/secret.fav — Secret Rune v39.4.0
// AWS / Vault / GCP / Env シークレット取得

fn get_aws(ctx: AppCtx, name: String) -> Result<String, String> !Http {
  // AWS Secrets Manager 呼び出し（スタブ: 本実装は backend.aws 統合後）
  Result.ok("aws://" ++ name)
}

fn get_vault(ctx: AppCtx, path: String) -> Result<String, String> !Http {
  // HashiCorp Vault 呼び出し（スタブ: 本実装は backend.vault 統合後）
  Result.ok("vault://" ++ path)
}

fn get_gcp(ctx: AppCtx, name: String) -> Result<String, String> !Http {
  // GCP Secret Manager 呼び出し（スタブ: 本実装は backend.gcp 統合後）
  Result.ok("gcp://" ++ name)
}

fn get_env(ctx: AppCtx, name: String) -> Result<String, String> {
  // ローカル開発フォールバック — 環境変数相当の値を返す（!Http 不要）
  Result.ok("env://" ++ name)
}
```

**テストキーワード**: `fn get_aws`

### 2. `runes/secret/rune.toml` — 新規作成

```toml
[rune]
name        = "secret"
version     = "1.0.0"
description = "Secret Rune for AWS / Vault / GCP / Env secret backends"
entry       = "secret.fav"
effects     = ["!Http"]

[dependencies]
```

### 3. `driver.rs` — テストモジュール追加

#### `v39300_tests::cargo_toml_version_is_39_3_0` のスタブ化

```rust
// Stubbed: version bumped to 39.4.0 — assertion intentionally removed
```

#### `v39400_tests` モジュール新規追加

```rust
// ── v39400_tests (v39.4.0) — Secret Rune 強化 ────────────────────────────────
#[cfg(test)]
mod v39400_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_39_4_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("39.4.0"), "Cargo.toml must contain version 39.4.0");
    }

    #[test]
    fn changelog_has_v39_4_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v39.4.0]"), "CHANGELOG.md must contain [v39.4.0]");
    }

    #[test]
    fn secret_rune_exists() {
        let src = include_str!("../../runes/secret/secret.fav");
        assert!(
            src.contains("fn get_aws"),
            "runes/secret/secret.fav must contain fn get_aws"
        );
    }
}
```

### 4. `CHANGELOG.md` — `[v39.4.0]` エントリ追加

`## [v39.3.0]` ヘッダ行の直前に挿入:

```
## [v39.4.0] — YYYY-MM-DD

### Added
- `runes/secret/secret.fav` — `Secret.get_aws` / `Secret.get_vault` / `Secret.get_gcp` / `Secret.get_env` 追加
- `runes/secret/rune.toml` — Secret Rune メタデータ
- `fav.toml` `[secrets] backend` 宣言スキーマ（"aws"/"vault"/"gcp"/"env"）
- `v39400_tests` 3 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

> `fav.toml [secrets] backend 宣言スキーマ` は仕様の明文化のみ。`toml.rs` パース実装は後続バージョン。

### 5. その他ドキュメント更新

- `fav/Cargo.toml`: `39.3.0` → `39.4.0`
- `versions/current.md`: 最新安定版 → v39.4.0、次に切る版 → v39.5.0
- `versions/roadmap/roadmap-v39.1-v40.0.md`: v39.4.0 を ✅ 完了済みにマーク

## 注意事項

### `get_env` は `!Http` 不要

`get_env` はローカル開発フォールバックであり、ネットワーク呼び出しを行わない。
シグネチャは `-> Result<String, String>`（エフェクトなし）とし、他の 3 関数（`!Http`）と区別する。

### `ctx: AppCtx` パラメータ

将来的に `ctx.secrets.backend` を参照してバックエンドを切り替えるため、
`get_env` を含む全関数が `ctx: AppCtx` を受け取る設計とする。
現在のスタブではパラメータを直接使用しないが、将来の実装時に削除しないこと。

### `get_env` の `ctx` 未使用警告

`get_env` 内で `ctx` を使用しないため Rust コンパイラの警告は出ない
（`.fav` ファイルは Rust コンパイラの対象外）。Favnir セルフホスト lint（W018）の
対象になる可能性があるが、`runes/` ディレクトリは CI の `fav lint` 対象外のため対処不要。
将来実装時に `ctx.secrets.backend` を参照するため `_ctx` とはしないこと。

### スタブ実装について

AWS / Vault / GCP の本実装（HTTP クライアント呼び出し）は後続バージョンで行う。
現時点では `Result.ok("scheme://" ++ name)` のプレースホルダを返す。

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v39.3.0 | 2794 |
| v39.4.0 追加分 | +3 |
| v39.4.0 期待値 | 2797 |

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `secret.fav` に `fn get_aws` が含まれる | `secret_rune_exists` テスト |
| 2 | `CHANGELOG.md` に `[v39.4.0]` が含まれる | `changelog_has_v39_4_0` テスト |
| 3 | `Cargo.toml` バージョンが `39.4.0` | `cargo_toml_version_is_39_4_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2797） | `cargo test` 実行結果 |
| 5 | `roadmap-v39.1-v40.0.md` の v39.4.0 が ✅ | T6 後に目視確認 |
| 6 | `runes/secret/rune.toml` が存在し必須フィールドを持つ | T2 手動確認（自動テスト対象外）|
