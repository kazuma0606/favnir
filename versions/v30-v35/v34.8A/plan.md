# v34.8A plan

## 実装順序

1. `fav/Cargo.toml` version を `35.3.0` → `35.4.0`
2. `fav/src/error_catalog.rs` — E0374 エントリ追加（E0373 の直後）
3. `fav/src/frontend/parser.rs` — `!` が戻り型後ろに来たら E0374 エラーを返すよう変更
4. `fav/src/lint.rs` — `check_w022_deprecated_effect_annotation` 関数 + `run_lint` 呼び出し行を削除
5. `fav/src/driver.rs` — スタブ化 + `v35400_tests` 追加
6. `cargo test` 全件 PASS 確認
7. CHANGELOG / benchmarks / current.md 更新
8. tasks.md COMPLETE

## parser.rs 変更の詳細

```
grep -n "parse_effects_acc\|parse_effects" fav/src/frontend/parser.rs
```
で変更対象の正確な行番号を確認してから実施する。

`peek_is_bang_effect` のような helper は不要。`self.peek()` で次のトークンが `!` であれば、
`parse_effects_acc` 呼び出し前にエラーを返す。

parser.rs の `parse_fn_def_after_ret` 内:
```rust
// v34.8A: !Effect syntax is removed
if self.current_token_is(Token::Bang) {
    return self.error_e0374("!Effect annotation syntax removed — use `ctx: AppCtx` parameter instead");
}
```

ただし `!` は他の文脈（否定演算子等）でも使われるため、
fn/stage の **戻り型直後** という文脈に限定してエラーを出す。
`parse_fn_def_after_ret` と `parse_stage_def` のエフェクト解析開始直前のみ変更する。

## v35400_tests の内容（5 件）

1. `cargo_toml_version_is_35_4_0` — バージョン確認
2. `effect_annotation_is_parse_error` — `!Http` を含む fn が E0374 を返す
3. `stage_effect_annotation_is_parse_error` — `!Db` を含む stage が E0374 を返す
4. `ctx_syntax_still_compiles` — `ctx: AppCtx` を使う fn が正常にコンパイルされる
5. `w022_lint_removed` — W022 が lint 出力に現れないことを確認

## 注意点

- `parser.rs` のテスト（行 3475, 3503, 3778）が `effects.contains(&Effect::Io)` を assert している。
  これらは v34.8A 後に必ず FAIL する。`effects` は空になるため、テストをスタブ化する。
- W022 テストが `driver.rs` に存在する場合、スタブ化する。
