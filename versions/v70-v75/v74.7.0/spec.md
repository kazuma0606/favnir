# v74.7.0 仕様書 — コミュニティ Rune 品質基準

Date: 2026-08-14

---

## Background

コミュニティが公開する Rune の品質を担保するため、`fav rune validate` コマンドの基盤を
`driver.rs` に実装する。Rune は rune.toml・実装ファイル・テスト・ドキュメント・サンプルの
5 項目を検証してスコアリング（100 点満点）を行い、80 点以上を公開要件とする。

本バージョンは検証ロジックのデータ構造と関数を `driver.rs` に実装する。
実際のファイルシステム走査・`fav publish rune` 時の自動検証フックは後続バージョンで対応する。

---

## Goals

1. `RuneValidationItem` 構造体（name / passed / message）を定義する
2. `RuneValidationReport` 構造体（rune_name / items / score）を定義する
3. `validate_rune_score(report: &RuneValidationReport) -> bool` — score >= 80 なら true
4. `format_rune_validation_report(report: &RuneValidationReport) -> String` — レポートをテキスト形式で返す
5. `v747000_tests` モジュール（2 件）を追加する
   - `rune_validate_scoring`
   - `rune_validate_min_score_enforced`

---

## API / コマンド例

```bash
$ fav rune validate ./runes/my-rune
✓ rune.toml: valid
✓ implementation: my-rune.fav (247 lines)
✓ tests: 3 test cases found
✓ documentation: README.md exists
⚠ No example .fav file found
Score: 85/100 (Publish requires >= 80)
```

### `RuneValidationItem` 構造体

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct RuneValidationItem {
    pub name: String,     // チェック項目名（例: "rune.toml"）
    pub passed: bool,     // 合格: true / 不合格: false
    pub message: String,  // 説明メッセージ（例: "valid"）
}
```

### `RuneValidationReport` 構造体

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct RuneValidationReport {
    pub rune_name: String,
    pub items: Vec<RuneValidationItem>,
    /// 0–100 の整数スコア。各項目が passed ならば均等配分で算出（呼び出し元が設定）
    pub score: u32,
}
```

### `validate_rune_score`

```rust
/// score >= 80 なら true（公開要件を満たす）
pub fn validate_rune_score(report: &RuneValidationReport) -> bool {
    report.score >= 80
}
```

### `format_rune_validation_report`

```rust
/// レポートをテキスト形式でフォーマットする
/// passed なら "✓"、false なら "⚠" プレフィックス
/// 末尾に "Score: {score}/100 (Publish requires >= 80)" を追記
pub fn format_rune_validation_report(report: &RuneValidationReport) -> String
```

---

## Success Criteria

1. `rune_validate_scoring` テストが pass する
   - `RuneValidationReport` を構築しスコア・項目数を assert
   - `format_rune_validation_report` の出力に "✓" と "⚠" が含まれることを assert
   - `format_rune_validation_report` の出力に "Score:" と "85"（スコア値）が含まれることを assert
   - `format_rune_validation_report` の出力に "80"（公開閾値）が含まれることを assert
2. `rune_validate_min_score_enforced` テストが pass する
   - score >= 80 のレポートで `validate_rune_score` が `true` を返すことを assert
   - score < 80 のレポートで `validate_rune_score` が `false` を返すことを assert
   - score == 80 のボーダーケースが `true` を返すことを assert
3. `cargo test` で 3684 tests pass（0 failures）

---

## スコープ外（明示的除外）

- `cmd_rune_validate(path)` 関数の実装（後続バージョンで対応）
- 実際のファイルシステム走査（rune.toml・実装ファイル・テスト・ドキュメントの読み込み）
- `fav publish rune` 時の自動 validate フック（後続バージョンで対応）
- `fav rune validate` の main.rs CLI エントリポイント（後続バージョンで対応）
- スコアリングアルゴリズムの自動計算（score は呼び出し元が設定）
- `site/` MDX 追加（v75.0.0 または後続フェーズで対応）
- MILESTONE.md 更新（宣言バージョンではないため不要）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `RuneValidationItem` / `RuneValidationReport` / `validate_rune_score` / `format_rune_validation_report` + `v747000_tests` 追加 |
| `fav/Cargo.toml` | `version = "74.7.0"` に更新 |
| `CHANGELOG.md` | v74.7.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョン・次に切る版を更新 |
