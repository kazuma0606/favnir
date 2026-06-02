# Favnir v9.11.0 Tasks

Date: 2026-06-02
Theme: LSP 補完 + go-to-definition 強化

---

## Phase A: ビルトイン関数テーブル定義

- [x] A-1: `BuiltinFn` 構造体を `completion.rs` に追加（`namespace` / `name` / `signature` / `params` フィールド）
- [x] A-2: `BUILTIN_FNS` 静的テーブルを定義（List / String / Map / Result / Option / IO / Json / Csv / Gen / Http / Llm / Env / Debug / Float / Int / Schema / T の主要関数）
- [x] A-3: `BUILTIN_NAMESPACES` 定数（補完判定に使う namespace 名の集合）

---

## Phase B: モジュール補完

- [x] B-1: `module_completions(ns: &str) -> Vec<CompletionItem>` を `completion.rs` に追加
  - `BUILTIN_FNS` を ns でフィルタして `CompletionItem { label, detail: signature, kind: Function }` に変換
- [x] B-2: `handle_completion` 内の `.` トリガー処理を拡張
  - カーソル前の識別子が `BUILTIN_NAMESPACES` に含まれる場合は `module_completions` を呼ぶ
  - それ以外は既存の `field_completions`（変更なし）
- [x] B-3: `cargo build` 通過確認

---

## Phase C: Rune 補完

- [x] C-1: `KNOWN_RUNES` 定数を `completion.rs` に追加（aws / cache / csv / db / email / fs / gen / graphql / grpc / http / json / llm / queue / slack / sql）
- [x] C-2: `rune_completions() -> Vec<CompletionItem>` を追加（各 Rune を `kind: Module` で返す）
- [x] C-3: `handle_completion` に `import rune "` パターンの判定を追加
  - カーソル前テキストが `import rune "` にマッチする場合 `rune_completions()` を返す
- [x] C-4: `protocol.rs` に `MODULE` completion kind 定数（= 9）を追加
- [x] C-5: `cargo build` 通過確認

---

## Phase D: Signature Help

- [x] D-1: `fav/src/lsp/signature.rs` を新規作成
  - `get_signature_help(src, position, symbols) -> Option<SignatureHelp>` を実装
  - `SignatureHelp` / `SignatureInformation` / `ParameterInformation` 型追加
  - ロジック: カーソルから逆スキャンして `(` を探す → 前の識別子を取得 → BUILTIN_FNS / ユーザー定義から候補を取得 → `,` カウントで `activeParameter` を決定
- [x] D-2: `mod.rs` に `textDocument/signatureHelp` ハンドラを追加
- [x] D-3: `mod.rs` の `initialize` レスポンスに `signatureHelpProvider: { triggerCharacters: ["(", ","] }` を追加
- [x] D-4: `mod.rs` に `mod signature;` と `use signature::get_signature_help;` を追加
- [x] D-5: `cargo build` 通過確認

---

## Phase E: 定義ジャンプ改善

- [x] E-1: Rune 関数ジャンプ対応 → スコープ外へ延期（定義ジャンプは doc.def_at を通じて実装されており、Rune ファイルパス解決は Rust 側の変更が必要なため v9.12.0 以降へ）
- [x] E-2: `seq` 内 stage 名ジャンプ対応 → スコープ外へ延期（既存の def_at ロジックで stage 定義へのジャンプは動作しているため優先度低）

---

## Phase F: テスト + self-check + バージョン更新 + commit

- [x] F-1: `v9110_tests` モジュールを追加（8 件）
  - [x] F-1a: `module_completion_list_contains_map_and_filter` — List completions に map/filter
  - [x] F-1b: `module_completion_string_contains_split_and_trim` — String completions に split/trim
  - [x] F-1c: `rune_completion_returns_known_runes` — http/csv/json が補完される
  - [x] F-1d: `signature_help_builtin_first_param` — `List.map(` → `activeParameter: 0`
  - [x] F-1e: `signature_help_builtin_second_param` — `List.map(xs,` → `activeParameter: 1`
  - [x] F-1f: `initialize_reports_signature_help_provider` — capability 確認
  - [x] F-1g: `builtin_namespaces_includes_list_and_string` — テーブル確認
  - [x] F-1h: `builtin_fns_table_has_entries` — テーブル構造確認
- [x] F-2: `cargo test v9110` — 8 件通過
- [x] F-3: `cargo test checker_fav_wire_self_check` — 通過
- [x] F-4: `cargo test bootstrap` — 通過
- [x] F-5: `cargo test` — 全件通過（1240 件）
- [x] F-6: `fav/Cargo.toml` version → `"9.11.0"`
- [x] F-7: `fav/self/cli.fav` の `run_version` → `"9.11.0"`
- [x] F-8: 本ファイル完了チェック
- [x] F-9: `memory/MEMORY.md` に v9.11.0 完了を記録
- [x] F-10: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `List.` の後にビルトイン関数補完が出る | ✓ |
| `String.` の後に関数補完が出る（型シグネチャ付き） | ✓ |
| `import rune "` の後に Rune 名補完が出る | ✓ |
| `foo(` の後に Signature help が表示される（ユーザー定義関数） | ✓ |
| `List.map(` の後に Signature help が表示される | ✓ |
| Rune 関数の定義ジャンプが `runes/<name>/<name>.fav` へ飛ぶ | → v9.12.0 延期 |
| `seq` 内の stage 名ジャンプが動作する | → v9.12.0 延期 |
| `cargo test v9110` — 8 件通過 | ✓ 8/8 |
| `cargo test checker_fav_wire_self_check` 通過 | ✓ |
| `cargo test bootstrap` 維持 | ✓ |

---

## スコープ外（将来版へ延期）

- Rune 関数の定義ジャンプ（`runes/<ns>/<ns>.fav` へ飛ぶ）
- `seq` 内 stage 名ジャンプ
- Tab 補完の使用頻度順ランキング
- `///` docstring の補完候補への組み込み
- インクリメンタルパース（補完レスポンスの高速化）
- `fav/src/lsp/` の Favnir 化
