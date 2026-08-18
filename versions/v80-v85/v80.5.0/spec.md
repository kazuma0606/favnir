# Spec: v80.5.0 — ステージ単体テスト（`StageTestCase`）

## Background

v80.1.0〜v80.4.0 でテストフレームワークの基盤（TestSuite / GoldenDataset / DataFactory / PropertyTest）を構築した。
本バージョンではパイプライン全体ではなく **個別ステージを単体テストする型** を追加する。
`StageInput` / `StageOutput` でステージの入出力を宣言し、`StageTestCase` と `run_stage_test` で期待出力と実際の出力を比較する。

ロードマップ: `versions/roadmap/roadmap-v80.1-v81.0.md`（v80.5.0 セクション）

> **テスト数補足**: ロードマップは 3817 + 2 = 3819 と記載しているが、
> v80.4.0 code-reviewer 対応で 3 件追加されたため実際のベースは **3821**。
> 本バージョンの完了条件は **3821 + 2 = 3823**。

## Goals

- `StageInput` / `StageOutput` 構造体を `test_framework.rs` に追加する
- `StageTestCase` 構造体を追加する
- `run_stage_test(test: &StageTestCase, actual: &StageOutput) -> TestCase` を実装する
- `format_stage_test_result(result: &TestCase) -> String` を実装する
- テスト 2 件を追加して **3823 tests** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

#[derive(Debug, Clone)]
pub struct StageInput {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct StageOutput {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct StageTestCase {
    pub stage_name: String,
    pub input: StageInput,
    pub expected: StageOutput,
}

/// StageTestCase を実行する。actual と expected の rows を行単位で比較し、
/// 一致すれば TestCase { status: Pass, message: None }、
/// 不一致なら TestCase { status: Fail, message: Some("row N differs: ...") } を返す。
pub fn run_stage_test(test: &StageTestCase, actual: &StageOutput) -> TestCase;

/// TestCase を人間が読める文字列に変換する。
/// - Pass: "PASS: <name>"
/// - Fail: "FAIL: <name> — <message>"
/// - Skip: "SKIP: <name>"
pub fn format_stage_test_result(result: &TestCase) -> String;
```

### `run_stage_test` の動作例

```rust
let input = StageInput {
    name: "load".to_string(),
    rows: vec![vec!["alice".to_string(), "30".to_string()]],
};
let expected = StageOutput {
    name: "transform".to_string(),
    rows: vec![vec!["alice".to_string(), "30".to_string()]],
};
let actual = StageOutput {
    name: "transform".to_string(),
    rows: vec![vec!["alice".to_string(), "30".to_string()]],
};
let test = StageTestCase { stage_name: "transform".to_string(), input, expected };
let result = run_stage_test(&test, &actual);
// result.status == TestStatus::Pass
// result.message == None
```

### `run_stage_test` の失敗例

```rust
let actual_wrong = StageOutput {
    name: "transform".to_string(),
    rows: vec![vec!["bob".to_string(), "25".to_string()]],
};
let result = run_stage_test(&test, &actual_wrong);
// result.status == TestStatus::Fail
// result.message == Some("row 0 differs: expected [\"alice\", \"30\"], got [\"bob\", \"25\"]")
// ↑ Rust {:?} フォーマット出力。実際の文字列は:
//   row 0 differs: expected ["alice", "30"], got ["bob", "25"]
```

## Success Criteria

- `cargo test` が **3823 tests**, 0 failures
- `stage_test_pass_when_output_matches`: 同一行 → Pass、message == None
- `stage_test_fail_when_output_differs`: 行 0 が異なる → Fail、message に差分情報を含む

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `StageInput` / `StageOutput` / `StageTestCase` / `run_stage_test` / `format_stage_test_result` |
| `fav/src/driver.rs` | 追記 | `mod v80500_tests`（テスト 2 件） |

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。

## Error Codes

新規エラーコードなし。

## 注記

- `run_stage_test` の戻り値型は既存の `TestCase`（`name: String`, `status: TestStatus`, `message: Option<String>`）を再利用する。
- `StageInput` と `StageOutput` は同一フィールド構成だが、型を分けることでステージ境界の意味論を明確にする。
- `GoldenDataset`（`name` / `rows`）と同一フィールドを持つが、用途が異なるため別型として定義する。`StageOutput` から `GoldenDataset` への変換は本バージョンのスコープ外とする（必要であれば v80.9.0 の安定化フェーズで検討する）。
- MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンでまとめて実施する。
