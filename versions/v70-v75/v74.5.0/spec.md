# v74.5.0 仕様書 — Pipeline Scheduling（`fav schedule`）

Date: 2026-08-14

---

## Background

データパイプラインを cron ベースで定期実行する `fav schedule` コマンドの基盤構造を
`driver.rs` に追加する。スケジュール設定の追加・一覧表示・即時実行に対応した
データ構造と関数を実装し、将来の永続化・デーモン化の土台を作る。

本バージョンは基盤データ構造と関数のみを実装する。`cmd_schedule_add` / `cmd_schedule_run`
関数・cron デーモン・`~/.fav_schedules.toml` への永続化・通知送信は後続バージョンで対応する。
`cmd_schedule_add` の前処理として cron 式バリデーション（`validate_cron_expr`）を先行実装する。

---

## Goals

1. `ScheduleEntry` 構造体（name / cron / pipeline / notify）を定義する
2. `validate_cron_expr(expr: &str) -> bool` — cron 式の基本バリデーションを実装する
3. `cmd_schedule_list(entries: &[ScheduleEntry]) -> String` — 一覧をフォーマットして返す
4. `v745000_tests` モジュール（2 件）を追加する
   - `schedule_add_parses_cron`
   - `schedule_list_returns_entries`

---

## API / コマンド例

```bash
$ fav schedule add daily-report \
    --cron "0 9 * * *" \
    --pipeline pipelines/daily_report.fav \
    --notify slack://my-channel

$ fav schedule list
NAME            CRON          LAST RUN              STATUS
daily-report    0 9 * * *     2026-08-08 09:00:02   OK
hourly-sync     0 * * * *     2026-08-08 10:00:01   OK

$ fav schedule run daily-report  # 即時実行
```

### `ScheduleEntry` 構造体

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduleEntry {
    pub name: String,
    pub cron: String,       // "0 9 * * *" 形式
    pub pipeline: String,   // パイプラインファイルパス
    pub notify: String,     // "slack://my-channel" 等（空文字列も許容）
}
```

### `validate_cron_expr`

```rust
/// cron 式を簡易バリデーションする（フィールド数のチェックのみ）
/// 有効な cron: スペース区切りで 5 フィールド
pub fn validate_cron_expr(expr: &str) -> bool {
    expr.split_whitespace().count() == 5
}
```

### `cmd_schedule_list`

```rust
/// スケジュール一覧をテキスト形式で返す
/// 例: "daily-report    0 9 * * *    pipelines/daily_report.fav"
/// 永続化実装前のため、エントリをメモリ上から受け取る設計とした
pub fn cmd_schedule_list(entries: &[ScheduleEntry]) -> String
```

---

## Success Criteria

1. `schedule_add_parses_cron` テストが pass する
   - `ScheduleEntry` を構築し、各フィールドが正しいことを assert
   - `validate_cron_expr("0 9 * * *")` が `true` を返すことを assert
   - `validate_cron_expr("invalid")` が `false` を返すことを assert
2. `schedule_list_returns_entries` テストが pass する
   - `cmd_schedule_list` の出力に各エントリの name / cron が含まれることを assert
   - 空スライスの場合に空文字列（または空行のみ）を返すことを assert
3. `cargo test` で 3680 tests pass（0 failures）

---

## スコープ外（明示的除外）

- `cmd_schedule_add` / `cmd_schedule_run` 関数の実装（後続バージョンで対応）
- 実際の cron デーモン・定期実行エンジン（後続バージョンで対応）
- `~/.fav_schedules.toml` への永続化（後続バージョンで対応）
- `--notify` の実際の通知送信（後続バージョンで対応）
- `site/` MDX 追加（後続バージョンで対応）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `ScheduleEntry` / `validate_cron_expr` / `cmd_schedule_list` + `v745000_tests` 追加 |
| `fav/Cargo.toml` | `version = "74.5.0"` に更新 |
| `CHANGELOG.md` | v74.5.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョン・次に切る版を更新 |
