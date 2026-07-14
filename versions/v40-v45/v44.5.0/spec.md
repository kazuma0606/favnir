# v44.5.0 Spec — Back-pressure x `fav policy` 統合

## 概要

パイプラインのバックプレッシャー制御として既に実装済みの `#[max_inflight(n)]` アノテーション（v42.5.0 で AST / パーサー追加）を活用し、**ステージ定義に付与された `max_inflight` 制約を収集** できるヘルパーを追加する。

`fav policy check --ci` コマンドでの policy 上限と各ステージ制約の照合、runtime 強制（VM レベル同時処理制限）は将来版のスコープ。本バージョンは **「`#[max_inflight(n)]` 付きステージの収集」AST レベル MVP** とする。

---

## AST 確認事項

- `TrfDef.max_inflight: Option<MaxInflightAnnotation>` — v42.5.0 で追加済み
- `MaxInflightAnnotation { n: u64, span: Span }` — 上限値と位置情報
- `TrfDef.name: String` — ステージ名
- `TrfDef.span: Span` — ステージ宣言全体の行番号
- `#[max_inflight(n)]` 構文 — parser 実装済み（`parse_max_inflight_annotation`）

---

## 機能詳細

### 1. `collect_stage_max_inflight_annotations` ヘルパー追加

`driver.rs` に以下の関数を追加:

```rust
pub fn collect_stage_max_inflight_annotations(src: &str, filename: &str) -> Vec<String>
```

- ソースを parse して `TrfDef` を走査
- `td.max_inflight.is_some()` のステージを収集
- 返り値: `"<filename>:<line>: <stage_name>: max_inflight=<n>"` 形式の文字列リスト

---

## テスト

`v44500_tests` 2 件:

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_44_5_0` | `Cargo.toml` に `"44.5.0"` が含まれる |
| `stage_max_inflight_annotation_detected` | `#[max_inflight(50)]` 付き stage が収集される（名前・値が含まれる） |

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2955 passed; 0 failed**（2953 + 2）
- `v44500_tests` 2 件 pass

---

## 注意事項

- `TrfDef.max_inflight` は `Option<MaxInflightAnnotation>` — `is_some()` で存在確認
- `MaxInflightAnnotation.n` は `u64` — `format!("max_inflight={}", ann.n)` で文字列化
- `TrfDef.span.line` を行番号として使用（`MaxInflightAnnotation.span.line` でも可だが、ステージ先頭行が自然）
- MVP: `FnDef` の `max_inflight` は存在しない（`TrfDef` のみ対象）
- `policy { max_inflight: N }` グローバルポリシーブロックの parse・VM 強制は将来版のスコープ（`ast.rs` に `PolicyBlock` / `PolicyDef` ノードが現時点で未定義のため実装不可）
- `parse_max_inflight_annotation` は `#[max_inflight(n)]` を直後の `TrfDef` に紐付ける — `TrfDef.max_inflight: Option<MaxInflightAnnotation>` として格納済み（parser の実装確認済み）
- ロードマップ推定（2944）は旧見積もり。実績 2953 を基準とする
- `v44400_tests::cargo_toml_version_is_44_4_0` をスタブ化すること
  - スタブ化の方法: `assert!` 行のみ削除し `// Stubbed: version bumped to 44.5.0 in v44.5.0.` に置き換える（`#[test]` アトリビュートと関数シグネチャは残す — テスト件数を変えないため）
