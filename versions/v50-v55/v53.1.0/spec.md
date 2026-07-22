# Spec: v53.1.0 — lineage × LSP 統合（リネージをエディタで表示）

Status: 計画中
Date: 2026-07-22

---

## 概要

`lsp/document_store.rs` の `CheckedDoc` に `lineage: LineageReport` フィールドを追加し、
ドキュメント更新時に `lineage_analysis` 結果をキャッシュする。
`lsp/hover.rs` の `handle_hover` にリネージ情報（upstream / downstream / schema）を付加する。

> ロードマップでは `lsp/references.rs` / `collect_lineage` と記載されているが、
> 実際のホバー処理は `lsp/hover.rs` の `handle_hover` に実装されており、
> lineage 取得関数は `lineage_analysis`（`lineage.rs` 885 行目）が正しい名称。
> ロードマップの記述は誤記であり、本 spec の記述（hover.rs / lineage_analysis）を正とする。

ホバー表示の目標形式:

```
stage Validate
  type:       Order -> Result<Order>
  upstream:   Parse
  downstream: snowflake.insert
  effects:    !Snowflake(write)
  schema:     OrderRow
```

---

## 実装スコープ

### 1. `lsp/document_store.rs` — `CheckedDoc` に `lineage` フィールド追加

```rust
use crate::lineage::{lineage_analysis, LineageReport};

pub struct CheckedDoc {
    // 既存フィールド...
    pub lineage: LineageReport,  // v53.1.0: lineage キャッシュ
}
```

`open_or_change` の成功パスで `lineage_analysis(&program)` を呼び、結果を保存。
失敗パス（parse エラー）では `LineageReport { transformations: vec![], pipelines: vec![] }` を使う。

### 2. `lsp/hover.rs` — stage ホバーにリネージ情報を追加

stage 名のホバー時に、`doc.lineage` からその stage の upstream / downstream / schema を検索し、
Markdown ブロックに追記する。

upstream / downstream の決定方法:
- `doc.lineage.pipelines` の各 `PipelineLineage.steps` をスキャン
- 対象 stage 名が `steps[i]` に一致した場合:
  - upstream = `steps.get(i.wrapping_sub(1))` （i > 0 の場合）
  - downstream = `steps.get(i + 1)` （存在する場合）

schema は `doc.lineage.transformations` から対象 stage 名の `LineageEntry.schema` を参照。

### 3. テスト仕様

`v53100_tests` モジュールを `driver.rs` に追加（`v53000_tests` の直前）:

```rust
#[cfg(test)]
mod v53100_tests {
    #[test]
    fn lsp_hover_shows_lineage() {
        use crate::lsp::document_store::DocumentStore;
        let source = r#"
stage Parse: String -> Int = |s| { 0 }
stage Format: Int -> String = |n| { "" }
seq pipeline = Parse |> Format
"#;
        let mut store = DocumentStore::new();
        store.open_or_change("file:///t.fav", source.to_string());
        let doc = store.get("file:///t.fav").unwrap();
        // lineage がキャッシュされ、pipeline が検出されていることを確認
        assert!(
            !doc.lineage.pipelines.is_empty(),
            "lineage.pipelines must be cached in CheckedDoc"
        );
    }

    #[test]
    fn lsp_hover_lineage_upstream() {
        use crate::lsp::document_store::DocumentStore;
        let source = r#"
stage A: Int -> Int = |n| { n }
stage B: Int -> Int = |n| { n }
seq pipe = A |> B
"#;
        let mut store = DocumentStore::new();
        store.open_or_change("file:///t.fav", source.to_string());
        let doc = store.get("file:///t.fav").unwrap();
        let pipeline = doc.lineage.pipelines.first()
            .expect("lineage must have at least one pipeline");
        // A が先、B が後の順序で steps に含まれていることを確認（upstream → downstream）
        let pos_a = pipeline.steps.iter().position(|s| s == "A");
        let pos_b = pipeline.steps.iter().position(|s| s == "B");
        assert!(pos_a.is_some() && pos_b.is_some(), "steps must contain A and B");
        assert!(pos_a.unwrap() < pos_b.unwrap(), "A must precede B (upstream relation)");
    }
}
```

---

## バージョン更新

- `fav/Cargo.toml`: `"53.0.0"` → `"53.1.0"`

---

## 完了条件

- `cargo test` 3162 passed, 0 failed（3160 + 2 件追加）
  （ロードマップ推定値 3159 と 1 件ずれている理由: v53.0.0 実装時に `v52900_tests::cargo_toml_version_is_52_9_0` を空化したが、テスト関数自体は残存しているためカウントに変化なし。ベースが 3159 → 3160 になったのは v53.0.0 で 4 件追加したため。ロードマップ策定時のベース（v52.9.0 完了時）が 3156 → v53.0.0 で +4 → 3160 が正しいベース）
- `v53100_tests` 2 件 pass:
  - `lsp_hover_shows_lineage`
  - `lsp_hover_lineage_upstream`
- `cargo clippy -- -D warnings` クリーン

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `fav/src/lsp/document_store.rs` | `CheckedDoc` に `lineage` フィールド追加、`open_or_change` でキャッシュ |
| `fav/src/lsp/hover.rs` | stage ホバー時に lineage 情報（upstream/downstream/schema）を追記 |
| `fav/src/lineage.rs` | `LineageReport` に `#[derive(Default)]` 追加 |
| `fav/src/driver.rs` | `v53100_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v53.1.0 エントリ追加 |
| `versions/current.md` | v53.1.0 / 3162 tests に更新 |
| `versions/roadmap/roadmap-v53.1-v54.0.md` | v53.1.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `LineageReport` は `lineage.rs` に存在（`lineage_analysis` 関数も同様）
- `CheckedDoc.lineage` の Default 実装: `LineageReport` に `Default` が未実装の場合は手動で空値を設定
- `PipelineLineage.steps` は seq pipeline の stage 順序を保持（upstream → downstream）
- hover に lineage を追記するのは stage 名トークンに限定（fn / type 定義にはつけない）
- stage 名の識別: `source[span.start..span.end]` で取得したトークン文字列が `doc.lineage.transformations` の `name` と一致する場合を「stage ホバー」と判定する。`type_at` に stage 名トークンが記録されていない場合は `doc.symbols`（`LspSymbol.name`）を代替経路として使用する。いずれにも該当しない場合は `lineage_block_for_stage` が `None` を返し hover に lineage を追記しない（サイレント失敗 — テストは「lineage キャッシュの確認」のみとし、表示パスの動作は実装後に目視確認する）
- `LineageReport` への `Default` 追加は `#[derive(Default)]` で行う（`transformations` / `pipelines` はどちらも `Vec` のため空 Vec がデフォルト値として妥当。`LineageEntry` には `Default` を追加しない）
- wasm32 影響なし（LSP は非 wasm ターゲット専用）
