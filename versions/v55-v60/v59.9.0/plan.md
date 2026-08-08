# v59.9.0 Plan — 安定化・コードフリーズ（Enterprise 1.0 前調整）

Date: 2026-07-30

---

## Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml`:
```
version = "59.8.0"  →  version = "59.9.0"
```

---

## Step 2: enterprise1-overview.mdx 拡充

`site/content/docs/enterprise/enterprise1-overview.mdx` の末尾に追記する（既存コンテンツはそのまま保持）。

以下の内容を**外側の ```` ```mdx ```` フェンスを除いて**ファイル末尾に追記する:

追記内容:

```mdx
## 認定手順

1. `fav.toml` に必要なセクションを追加する（[設定チェックリスト](../cookbook/enterprise-checklist)参照）
2. `fav certify --level enterprise` を実行して全要件を確認する
3. 生成された `enterprise-cert.json` を CI アーティファクトとして保存する
4. `fav migrate --from v1 --to enterprise --dry-run` で既存コードを確認する
5. `fav migrate --from v1 --to enterprise --in-place <file>` で自動修正を適用する

## クイックスタート

```toml
# fav.toml — Enterprise 1.0 最小構成
[security.rbac]
enabled = true

[secrets]
provider = "aws-secrets-manager"

[security.tls]
enabled = true

[sla]
latency_p99_ms   = 200
availability_pct = 99.9
```
```

---

## Step 3: driver.rs — v59900_tests 追加

**注意**: Step 2（MDX 拡充）を先に完了させること（`include_str!` はコンパイル時に読み込む）。

既存の `// ─────────` セパレータ行（`v59800_tests` ブロックの直前）の**前**に挿入する。
パターン: セパレータ行 → 空行 → `// -- v59900_tests ...` コメント → モジュール本体。

```rust
// -- v59900_tests (v59.9.0) -- 安定化・コードフリーズ --
#[cfg(test)]
mod v59900_tests {
    #[test]
    fn cargo_toml_version_is_59_9_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"59.9.0\""),
            "Cargo.toml version should be 59.9.0"
        );
    }

    #[test]
    fn enterprise1_overview_doc_complete() {
        let content = include_str!(
            "../../site/content/docs/enterprise/enterprise1-overview.mdx"
        );
        assert!(
            content.contains("認定手順"),
            "enterprise1-overview.mdx should contain '認定手順' section"
        );
        assert!(
            content.contains("クイックスタート"),
            "enterprise1-overview.mdx should contain 'クイックスタート' section"
        );
    }
}
```

**注意**: `cargo_toml_version_is_59_9_0` は rolling check パターンを採用する。
v60.0.0 以降、このテストは既存の 7 件と同様に assertion と failure メッセージが更新される（rolling check プールが 8 件になる）。

---

## Step 4: driver.rs — ローリングチェック更新

`"59.8.0"` → `"59.9.0"` に一括更新（assertion 7 件 + failure メッセージ 7 件）。

全 7 件とも同一パターン:
- assertion: `contains("version = \"59.8.0\"")` → `contains("version = \"59.9.0\"")`
- failure msg: `"Cargo.toml version should be 59.8.0"` → `"Cargo.toml version should be 59.9.0"`

**注意**: コメント行（`// -- v59800_tests (v59.8.0) --` 等）の `59.8.0` は置換しないこと。
引用符付き文字列 `version = \"59.8.0\"` および `"Cargo.toml version should be 59.8.0"` のみを対象とすること。
**注意**: `v59100_tests`〜`v59800_tests` は rolling check なし → 変更不要。

---

## Step 5: テスト実行

```bash
cargo test -j 8 -- --test-threads=8
```

確認事項:
- `v59900_tests::cargo_toml_version_is_59_9_0` pass
- `v59900_tests::enterprise1_overview_doc_complete` pass
- 総テスト数 **3326** tests passed, 0 failed

---

## Step 6: 事後処理

- `CHANGELOG.md` に v59.9.0 エントリを追加
- `versions/current.md` を v59.9.0 / 3326 tests に更新
- `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.9.0 実績欄更新（Step 5 完了後に実施）
- `versions/roadmap/roadmap-v59.1-v60.0.md` の v60.0.0 ベース数を `3326` に確定（Step 5 完了後に実施）
- `versions/roadmap/roadmap-v59.1-v60.0.md` の v60.0.0 完了条件テスト数を `3326 + 4 = 3330` に更新（`≥ 3316` / `ベース 3312 + 4 = 3316` → `≥ 3330` / `ベース 3326 + 4 = 3330`）
- `versions/v55-v60/v59.9.0/tasks.md` を COMPLETE ステータスに更新
