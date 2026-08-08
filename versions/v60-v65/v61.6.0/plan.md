# v61.6.0 実装計画

## フェーズ

### Phase 1: error_catalog.rs 更新
- E0009 の `long_description` を差分表示に対応したテキストに更新

### Phase 2: checker.rs — `diff_types` 追加 + E0009 call site 更新
1. `diff_types(expected, found, type_defs) -> Option<String>` をプライベート関数として追加
2. `check_stage_output` 付近の E0009 発行箇所を `type_error_h(..., hints)` に切り替え
   - `diff_types` の結果を `hints` に追加

### Phase 3: driver.rs — テスト追加
- `v61500_tests` モジュールが driver.rs に存在することを grep で確認
- `v61600_tests` モジュールを `v61500_tests` の直前に追加
- `type_error_diff_display_record`: Record vs スカラーで hint が出ることを確認
- `type_error_suggestion_e0009`: E0009 の suggestion テキスト確認

### Phase 4: ビルド・テスト
- `cargo build` でコンパイルエラーがないことを確認
- `cargo test` で 3371 tests passed, 0 failed を確認

## 実装順序の根拠

- AST 変更なし → exhaustive match 更新不要
- `diff_types` は新規追加のみ（既存ロジック非破壊）
- E0009 call site の変更は 1〜2 箇所に限定

## リスク

- `type_defs` の実際のフィールド名・型が spec.md の仮定と異なる可能性
  → Phase 2 開始時に checker.rs の `Checker` 構造体定義を確認する
- `ty_to_str` の実際の関数名が異なる可能性
  → grep で確認してから使う
