# Favnir v8.4.0 実装計画

Date: 2026-05-30

---

## Phase A: `check_file_with_fav` ヘルパー追加（driver.rs）

`check_single_file` の内部ロジック（checker.fav パス）を
`cmd_run_self_hosted` から呼べる形に切り出す。

**変更ファイル**: `src/driver.rs`

追加する関数:
```rust
fn check_file_with_fav(path: &str) -> Result<(), Vec<TypeError>> {
    let source = load_file(path);
    let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
        eprintln!("{}", e); process::exit(1);
    });
    let prog_vm = crate::middle::ast_lower_checker::lower_program(&program);
    crate::checker_fav_runner::run_checker_fav(prog_vm)
        .map_err(crate::checker_fav_runner::msgs_to_type_errors)
}
```

エラー表示は呼び出し側（`cmd_run_self_hosted`）で行う。

---

## Phase B: `cmd_run_self_hosted` の型チェック切替

`load_and_check_program(file)` の呼び出しを削除し、以下に置き換える:

1. `find_entry(file)` でファイルパスを解決
2. `check_file_with_fav(&source_path)` で checker.fav 型チェック
3. エラーがあれば `format_diagnostic` で表示して exit
4. 残り（compiler.fav コンパイル → deserialize → 実行）は変更なし

**変更ファイル**: `src/driver.rs`

---

## Phase C: 統合テスト追加

`run_self_hosted_tests` モジュールに以下を追加:

- `run_self_hosted_type_error_caught` — 型エラーのある .fav を `--self-host` で実行すると
  checker.fav が E0xxx を報告して exit する（`#[should_panic]` または戻り値 Err で検証）
- `run_self_hosted_match_option` — `Option<Int>` を match するコードが正しく実行される

合わせて既存 5 件が引き続き通ることを確認する。

---

## Phase D: 最終確認

1. `cargo test` — 全テスト通過
2. `fav run --self-host fav/self/stdlib/list_stdlib.fav` 等で手動確認（任意）
3. tasks.md を完了状態に更新
4. commit

---

## 考慮事項

### `find_entry` の戻り値

`find_entry(file)` は `(String, Option<(FavToml, PathBuf)>)` を返す。
`cmd_run_self_hosted` は今後も `source_path` のみ使うのでタプルは `let (source_path, _) =` で受ける。

### `format_diagnostic` のシグネチャ

`format_diagnostic(source: &str, error: &TypeError) -> String`
`source` が必要なので `load_file` した内容を保持しておく。

### checker.fav の制限

checker.fav は単一ファイルをチェックする。rune import を含むファイルは
現状 checker.fav では完全チェックできない（rune モジュールの型情報なし）。
`--self-host` でこのケースを踏んだ場合は checker.fav がエラーを出す可能性あり。
v8.4.0 では limitation として許容し、ドキュメントに記載する。

### スタックサイズ

`run_checker_fav` は `cmd_check` の `self_check` テストで 64MB スタックスレッドを使う。
通常の `cmd_run_self_hosted` 呼び出しでスタックオーバーフローが起きる場合は
同様のスタック設定が必要。v8.4.0 のテスト対象は小さいファイルなので許容範囲内の想定。
