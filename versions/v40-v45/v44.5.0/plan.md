# v44.5.0 Plan — Back-pressure x `fav policy` 統合

## 前提

- 現行バージョン: `44.4.0`（2953 tests）
- 追加テスト数: 2 件
- 目標テスト数: 2955
- ロードマップ推定（2944）は旧見積もり。実績 2953 を基準とする

---

## AST 確認事項（実装前確認済み）

- `TrfDef.max_inflight: Option<MaxInflightAnnotation>` — v42.5.0 で追加済み（ast.rs 行 659）
- `MaxInflightAnnotation { n: u64, span: Span }` — 上限値 `n` と位置情報
- `TrfDef.name: String` — ステージ名
- `TrfDef.span: Span` — ステージ宣言全体の行番号（`td.span.line`）
- `#[max_inflight(n)]` 構文 — parser に `parse_max_inflight_annotation` 実装済み（parser.rs 行 592 付近）。アノテーションは直後の `stage` キーワードの `TrfDef` に紐付けられ、`td.max_inflight` に格納される
- `Item::TrfDef(td)` でマッチ
- `ast.rs` に `PolicyBlock` / `PolicyDef` ノードは未定義 — `policy { ... }` ブロック parse は将来版のスコープ

---

## ステップ

### Step 1: driver.rs — `collect_stage_max_inflight_annotations` 追加

`collect_annotated_lineage_bindings` の直後（`bare_inner_literal_line` の直前）に配置:

```rust
/// v44.5.0: `#[max_inflight(n)]` アノテーション付きステージを収集。
/// Back-pressure policy 照合の AST レベル MVP。
/// NOTE: FnDef には max_inflight フィールドが存在しない（TrfDef のみ対象）。
/// NOTE: policy { max_inflight: N } グローバルポリシーブロック parse・VM 強制は将来版のスコープ。
pub fn collect_stage_max_inflight_annotations(src: &str, filename: &str) -> Vec<String> {
    use crate::ast::Item;

    let program = match crate::frontend::parser::Parser::parse_str(src, filename) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let mut result = Vec::new();

    for item in &program.items {
        if let Item::TrfDef(td) = item {
            if let Some(ann) = &td.max_inflight {
                result.push(format!(
                    "{}:{}: {}: max_inflight={}",
                    filename,
                    td.span.line,
                    td.name,
                    ann.n,
                ));
            }
        }
    }

    result
}
```

### Step 2: driver.rs — `v44500_tests` 追加 / スタブ化 / Cargo.toml

`v44400_tests` の直前に挿入:

```rust
// -- v44500_tests (v44.5.0) -- Back-pressure x fav policy 統合 --
#[cfg(test)]
mod v44500_tests {
    #[test]
    fn cargo_toml_version_is_44_5_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"44.5.0\""), "Cargo.toml version mismatch");
    }
    #[test]
    fn stage_max_inflight_annotation_detected() {
        let src = r#"
#[max_inflight(50)]
stage Process: List<Int> -> List<Int> = |items| {
  bind result: List<Int> <- items
}
"#;
        let entries = super::collect_stage_max_inflight_annotations(src, "v44500_test.fav");
        assert!(!entries.is_empty(), "expected max_inflight annotation, got: {:?}", entries);
        assert!(
            entries.iter().any(|e| e.contains("Process") && e.contains("max_inflight=50")),
            "expected 'Process: max_inflight=50' in entries: {:?}", entries
        );
    }
}
```

スタブ化: `v44400_tests::cargo_toml_version_is_44_4_0` の `assert!` 行のみを削除し、以下に置き換える（`#[test]` アトリビュートと関数シグネチャは残す）:

```rust
// Stubbed: version bumped to 44.5.0 in v44.5.0.
```

`fav/Cargo.toml` version: `44.4.0` → `44.5.0`

### Step 3: CHANGELOG.md に v44.5.0 エントリ追加

### Step 4: テスト実行（2955 passed; 0 failed）

### Step 5: バージョン管理ドキュメント更新

- `versions/current.md` → v44.5.0、2955 tests、次版 v44.6.0
- `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.5.0 を `✅ COMPLETE`
- `versions/v40-v45/v44.5.0/tasks.md` → COMPLETE

---

## 注意事項

- `TrfDef.max_inflight` は `Option<MaxInflightAnnotation>` — `if let Some(ann) = &td.max_inflight` でマッチ
- `ann.n` は `u64` — `{}` フォーマットで文字列化可
- `td.span.line` を行番号として使用（ステージ宣言先頭行）
- v44.4.0 テストソースの `Stream<Float>` → `List<Float>` 修正は実施済み（2953 tests 確認済み）
