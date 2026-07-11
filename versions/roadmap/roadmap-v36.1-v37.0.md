# Roadmap v36.1.0 〜 v37.0.0 — Data Quality First

Date: 2026-07-06
Status: 骨格確定（v35.0 完了時点）、詳細は v36.0 完了後に確定

---

## 目標

v36.0「Deployment Story」で「自動デプロイできる」を実現した。
このフェーズは **「データ品質を型で保証できる」** を実現する。

> **Data Quality First の定義**
> 「`schema` キーワードでテーブル/列スキーマを型定義し（v32.4 基盤を拡張）、
>  `expect` ブロックで品質ルールを宣言的に記述できる。
>  `fav validate` で CSV/Parquet のスキーマ検証が実行でき、
>  スキーマ不整合は W025 lint ルールとして静的に検出される」

**前版との関係**:
- v32.4: `schema "postgres:users"` — 文字列リテラルからの外部スキーマ参照 ✓
- v36.x: `schema Orders { id: Int, ... }` **インライン定義構文** と `expect` 品質ブロックを追加（新規）

---

## バージョン計画

### v36.1.0 — `schema` リテラル定義構文

**前版確認**: v32.4 で文字列リテラル形式 `schema "postgres:..."` は実装済み。
v36.1 はインライン構造体形式を追加する。

**想定構文**:
```favnir
schema Orders {
  id: Int
  customer_id: Int
  amount: Float where { amount >= 0.0 }
  status: String where { status in ["pending", "shipped", "delivered"] }
  created_at: DateTime
}
```

**完了条件**: `schema Name { ... }` 構文が型チェックを通る / Rust テスト 3 件

---

### v36.2.0 — `expect` ブロック

**想定構文**:
```favnir
fn validate_orders(rows: List<Orders>) -> Result<List<Orders>, String> {
  expect rows {
    not_empty
    all(|r| r.amount >= 0.0)
    no_nulls([.customer_id, .amount])
    unique([.id])
  }
}
```

**完了条件**: `expect` ブロックが型チェックと実行を通る / Rust テスト 3 件

---

### v36.3.0 — W025 `schema_mismatch` lint ルール

`lint.rs` に W025 追加。`schema` 定義と使用箇所の型が一致しない場合に警告。

**完了条件**: `fav lint` で W025 が報告される / Rust テスト 2 件

---

### v36.4.0 — `fav validate` コマンド ✅

`fav validate --schema orders.fav data.csv` で CSV/Parquet のスキーマ検証を実行する。

**完了条件**: `fav validate` コマンドが動作する / Rust テスト 2 件（実装: 5 件、2676 tests pass）

---

### v36.5.0 — Data Contract 規約 ✅

- `contracts/` ディレクトリ規約策定
- `fav new --template data-contract` テンプレート追加
- `fav contract check` コマンド

**完了条件**: Rust テスト 2 件（実装: 5 件、2681 tests pass）

---

### v36.6.0 — E0380〜E0384 スキーマ不整合エラーコード ✅

| コード | 意味 |
|---|---|
| E0380 | `schema_field_missing` |
| E0381 | `schema_type_mismatch` |
| E0382 | `schema_constraint_violated` |
| E0383 | `schema_duplicate_key` |
| E0384 | `schema_extra_field` |

**完了条件**: `error_catalog.rs` に定義済み / Rust テスト 2 件

---

### v36.7.0 — Great Expectations 互換エクスポート ✅

`fav validate --export ge --output suite.json` で Great Expectations 形式に出力する。

**完了条件**: Rust テスト 1 件

---

### v36.8.0 — `fav schema diff` ✅

`fav schema diff old.fav new.fav` でスキーマ変更差分と後方互換性チェックを表示する。

**完了条件**: Rust テスト 2 件

---

### v36.9.0 — v37.0 前調整・安定化 ✅

---

### v37.0.0 — Data Quality First マイルストーン宣言 ★クリーンアップ ✅

**宣言文（暫定）**:

> 「`schema` でテーブル/列の型と制約を宣言し、
>  `expect` でビジネスルールをパイプラインに埋め込み、
>  `fav validate` でデータを検証できる。
>  スキーマ不整合は W025 lint で静的に検出され、
>  違反は E0380〜E0384 として報告される。
>
>  これが Favnir v37.0 — Data Quality First の姿である。」

**完了条件**:
- v36.1〜v36.9 の全機能が動作する / テスト数 4000+（実測: 2703 件、4000+ は後続スプリントへ持ち越し）
- GitHub Issues の P1/P2 ラベル付きオープンバグが **0 件**（OSS 公開前のため GitHub Issues 未開設、対象外）
- `★クリーンアップ` 完了 ✅（cargo clean 実施）

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v35.1-v40.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v35.1-v36.0.md`
- 次サブスプリント: `versions/roadmap/roadmap-v37.1-v38.0.md`
