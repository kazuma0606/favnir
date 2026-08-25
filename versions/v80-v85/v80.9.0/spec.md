# Spec: v80.9.0 — 安定化・コードフリーズ（Test-Driven Data 1.0 完成宣言）

## Background

v80.1.0〜v80.8.0 で Test-Driven Data 1.0 フレームワークを段階的に構築した。
本バージョンでは新機能の追加を行わず、v80.1.0〜v80.8.0 の全実装が統合された状態で
安定動作することを確認し、**Test-Driven Data 1.0 完成宣言**とする。

ロードマップ: `versions/roadmap/roadmap-v80.1-v81.0.md`（v80.9.0 セクション）

> **テスト数補足**: ロードマップは 3825 + 2 = 3827 と記載しているが、
> v80.2.0〜v80.8.0 の code-reviewer 対応で累積 9 件追加されたため実際のベースは **3835**。
> （内訳: v80.8.0 完了時 3834 tests + v80.8.0 code-reviewer 追加 1 件 = 3835）
> （累積 9 件の内訳: v80.4.0 +3、v80.5.0 +2、v80.6.0 +1、v80.7.0 +2、v80.8.0 +1）
> 本バージョンの完了条件は **3835 + 2 = 3837**。

> **スコープ補足**: 本バージョンは安定化のみ。新しい型・関数・エラーコードは追加しない。
> `cmd_test` への `--format` オプション追加は v80.9.0 以降に持ち越す（ロードマップ記載の通り）。
> `fav test` E2E チェック（ロードマップ行 192）は Rust 単体テスト内で `format_test_summary`
> フローを呼ぶことで代替する（CLI 統合テストは本バージョンのスコープ外）。

## Goals

- v80.1.0〜v80.8.0 の全機能が統合された状態で動作することを確認する統合テスト 2 件を追加する
- **3837 tests** を達成する
- 新機能・新 API は一切追加しない

## API / Type Definitions

新規型・関数なし。

### 統合テスト内容

```rust
// mod v80900_tests in fav/src/driver.rs

// test_framework_full_sprint_all_stable:
//   v80.1〜v80.8 の全型（TestSuite / DataFactory / PropertyTest / StageTestCase /
//   TestCoverageReport / SchemaSnapshot / TestReport）を
//   それぞれ 1 インスタンス生成し、パニックしないことを確認する。
//   戻り値のアサートは最小限（型が存在し呼び出せること）。

// test_framework_e2e_pipeline_tested:
//   DataFactory → StageTestCase → TestSuite → TestReport → format_test_summary の
//   End-to-End フローを実行し、summary 文字列に suite name が含まれることを確認する。
```

## Success Criteria

- `cargo test` が **3837 tests**, 0 failures
- `fav test` CLI E2E は本バージョンでは Rust 単体テストで代替（CLI smoke test はスコープ外）
- `test_framework_full_sprint_all_stable`:
  - `TestSuite` / `DataFactory` / `PropertyTest` / `StageTestCase` /
    `TestCoverageReport` / `SchemaSnapshot` / `TestReport` の各型を生成できる
  - 各関数呼び出しがパニックしない
- `test_framework_e2e_pipeline_tested`:
  - `DataFactory::from_seed(1)` → `generate_rows` → `StageOutput` 生成 →
    `run_stage_test` → `TestSuite` → `TestReport` → `format_test_summary` の
    フローが動作する
  - `format_test_summary` の出力に `"pipeline_tests"` が含まれる

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 追記 | `mod v80900_tests`（テスト 2 件） |

> `test_framework.rs` / `lib.rs` への変更は不要。

## Error Codes

新規エラーコードなし。

## 注記

- 本バージョンは安定化フェーズ。バグ修正のみ許可。
- `cmd_test` への `--format` オプション追加は v80.9.0 以降のスコープ（本バージョン外）。
- MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンでまとめて実施する。
