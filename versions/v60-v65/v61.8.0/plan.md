# v61.8.0 実装計画

## フェーズ構成

| Phase | 作業 | ファイル |
|---|---|---|
| 1 | `LintConfig` + `lint_program_with_config` 追加 | `lint.rs` |
| 2 | `LintTomlConfig.strict` フィールド追加 | `toml.rs` |
| 3 | `cmd_lint` に `strict` 引数追加 | `driver.rs` |
| 4 | `fav lint --strict` フラグ追加 | `main.rs` |
| 5 | `v61800_tests` 追加 | `driver.rs` |
| 6 | ビルド・テスト確認 | — |
| 7 | ドキュメント更新 | roadmap / current.md / CHANGELOG |

---

## Phase 1: lint.rs — LintConfig 追加

`pub struct LintConfig` を `lint_program` の直前に追加。
`lint_program_with_config(program, config)` を `lint_program` の直後に追加。
strict=true の場合、W040 コードのメッセージ末尾に ` [strict]` を付与。

**ポイント**: `lint_program` は変更しない（後方互換のため）。

---

## Phase 2: toml.rs — LintTomlConfig.strict 追加

`LintTomlConfig` に `pub strict: Option<bool>` を追加。
`parse_lint_config` 関数（または同等のパース処理）でキー `"strict"` を検出し `bool` にパース。

---

## Phase 3: driver.rs — cmd_lint 更新

`cmd_lint` シグネチャに `strict: bool` を追加。
内部で `LintConfig { strict, perf: false }` を構築し `lint_program_with_config` を呼ぶ。
`lint_program` の直接呼び出し箇所を `lint_program_with_config` に切り替える。

`cmd_check` 内には現時点で `lint_program` の直接呼び出しが存在しない（W006 処理のみ）。
そのため「切り替え」ではなく、`cmd_check` 内の適切な位置（`strict` フラグ処理ブロックの後）に
`lint_program_with_config(&program, &LintConfig { strict, perf: false })` を**新規追加**する。
プログラムは既に `check_single_file` でパース済みのため `Parser::parse_str` の再実行は不要。

---

## Phase 4: main.rs — fav lint --strict 追加

`fav lint` のフラグ解析ループに `"--strict"` を追加（`strict` 変数を宣言・設定）。
`cmd_lint(file, warn_only, deny, allow, strict)` の呼び出しに `strict` を追加。

---

## Phase 5: driver.rs — v61800_tests 追加

`v61700_tests` の直前に `v61800_tests` モジュールを挿入。

テスト 1: `check_strict_mode_w040_tagged`
- `fn f(x: Int) -> _ { x }` を `lint_program_with_config` に `strict=true` で渡す
- W040 の message に `"[strict]"` が含まれることを確認

テスト 2: `fav_toml_lint_strict`
- `strict = true` を含む `[lint]` セクション文字列を `parse_fav_toml` でパース
- `LintTomlConfig.strict == Some(true)` であることを確認

---

## Phase 6: ビルド・テスト

```bash
cargo build            # エラー 0
cargo test v61800      # 2 件 PASS
cargo test -j 8 -- --test-threads=8   # 3376 tests passed, 0 failed
```

---

## Phase 7: ドキュメント更新

- `versions/roadmap/roadmap-v61.1-v62.0.md` — v61.8.0 実績を記録
- `versions/current.md` — 進行中を v61.8.0、次を v61.9.0 に更新
- `CHANGELOG.md` — v61.8.0 エントリ追加
- `tasks.md` を COMPLETE に更新
