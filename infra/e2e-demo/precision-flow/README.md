# Precision & Flow E2E Demo

Favnir v44.6.0 — CEP + Refinement type + Opaque type + Policy gate（governance 制御）の統合デモ。

## パイプライン概要

```
Kafka → IngestEvents → DetectHighValue → PolicyGate
```

## 機能一覧

| 機能 | 実装箇所 |
|---|---|
| Refinement type | `type HighValue = Float where \|v\| v > 1000.0` |
| Opaque type | `opaque type OrderId = String` |
| CEP pattern | `cep pattern HighValueDetected { HighValue within 300 }` |
| Back-pressure governance | `#[max_inflight(50)] stage PolicyGate` |
| 型注釈付き bind | `bind detected: List<HighValue> <- events` |

## 実行方法（将来版）

```bash
fav run src/demo.fav
```

## 関連バージョン

- v44.1.0 — Refinement type × Streaming 統合
- v44.2.0 — CEP × Refinement type
- v44.3.0 — Stream join × Opaque type
- v44.4.0 — 型推論 × パイプライン lineage
- v44.5.0 — Back-pressure × `fav policy` 統合
- v44.6.0 — Precision & Flow E2E デモ（本バージョン）
