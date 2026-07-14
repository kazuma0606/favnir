# v44.6.0 Spec — Precision & Flow E2E デモ

## 概要

v44.1〜v44.5 で実装した Precision & Flow 機能群（Refinement type × Streaming、CEP × Refinement type、Opaque type、型注釈 lineage、`#[max_inflight]` back-pressure）を統合した **E2E デモパイプライン** を `infra/e2e-demo/precision-flow/` に追加する。

デモは `Kafka → CEP → Opaque join → Policy gate` の完全パイプラインを Favnir ソースで表現し、Precision & Flow 各機能の実用的な組み合わせを示す。ロードマップ記載の「governance」は `#[max_inflight]` + Policy gate による back-pressure ガバナンス制御を指す。

---

## 成果物

### `infra/e2e-demo/precision-flow/src/demo.fav`

以下の機能を含む Favnir パイプライン:

- `type HighValue = Float where |v| v > 1000.0` — Refinement type
- `opaque type OrderId = String` — Opaque type
- `cep pattern HighValueDetected { HighValue within 300 }` — CEP
- `#[max_inflight(50)] stage PolicyGate: ...` — Back-pressure ポリシー
- 各ステージに型注釈付き `bind` 束縛

### `infra/e2e-demo/precision-flow/README.md`

デモの概要・実行方法・各ステージの説明を記載した Markdown ドキュメント。

---

## テスト

`v44600_tests` 1 件:

| テスト名 | 内容 |
|---|---|
| `precision_flow_e2e_demo_structure` | `infra/e2e-demo/precision-flow/` ディレクトリ・`src/demo.fav`・`README.md` の存在確認 |

テストパターンは `v10900_tests::snowflake_e2e_demo_structure` と同形式（`CARGO_MANIFEST_DIR` → `infra/e2e-demo/precision-flow/` パス）。

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2956 passed; 0 failed**（2955 + 1）
- `v44600_tests` 1 件 pass
- `infra/e2e-demo/precision-flow/src/demo.fav` が存在する
- `infra/e2e-demo/precision-flow/README.md` が存在する

---

## 注意事項

- `demo.fav` は実行可能な Favnir コードである必要はない（E2E デモの構造確認が目的。rune import 解決・VM 実行は将来版のスコープ）
- テストは `include_str!` ではなく `std::path::Path::exists()` を使用（他の E2E デモテストと同形式）
- `v44500_tests::cargo_toml_version_is_44_5_0` をスタブ化すること
  - スタブ化の方法: `assert!` 行のみ削除し `// Stubbed: version bumped to 44.6.0 in v44.6.0.` に置き換える（`#[test]` アトリビュートと関数シグネチャは残す）
- ロードマップ推定（2945）は旧見積もり。実績 2955 を基準とする
