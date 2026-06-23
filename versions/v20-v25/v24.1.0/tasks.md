# v24.1.0 — 形式的仕様書生成（`fav spec`）タスク

## ステータス: COMPLETE（2026-06-23）

---

## タスク一覧

### T0: 事前確認

- [x] `grep -n "version = " fav/Cargo.toml` — `"24.0.0"` であること
- [x] `grep -n "mod v240000_tests\|mod v241000_tests" fav/src/driver.rs | head -5` — v241000_tests 未存在
- [x] `grep -n "cmd_spec\|\"spec\"" fav/src/driver.rs fav/src/main.rs | head -5` — 全 0 件
- [x] `grep -n "fav spec" README.md | head -3` — 0 件であること

---

### T1: `fav/src/driver.rs` — `cmd_spec` 追加

- [x] **T1-1**: `SPEC_CONTENT` 定数（raw string）を `md_to_html`（line 10244）の**直後**（`build_nav_html` の直前）に追加
  - セクション 1（型システム）に `"型システム"` 文字列を含む
  - セクション 2（opcode）に `"0x01"` を含む
  - セクション 3（エフェクトシステム）を含む
  - セクション 4（パターンマッチ）を含む
- [x] **T1-2**: `fn build_spec_markdown() -> String` を `SPEC_CONTENT` の直後に追加
  - `format!("# Favnir 言語仕様書\n\nバージョン: {}\n\n---\n\n", env!("CARGO_PKG_VERSION"))` でヘッダ生成
  - `format!("{}{}", header, SPEC_CONTENT)` で結合
- [x] **T1-3**: `pub fn cmd_spec(format: &str) -> String` を `build_spec_markdown` の直後に追加
  - `"html"` → `md_to_html(&md)`
  - それ以外 → `md`（Markdown そのまま）
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T2: `fav/src/main.rs` — `"spec"` サブコマンド追加

- [x] `Some("doc") => { ... }` ブロックの直後、`Some("transpile")` の前に `Some("spec")` アームを追加
  - `args.iter().position(|a| a == "--format")` で format フラグを検索
  - 未指定時は `"markdown"` をデフォルト
  - `println!("{}", driver::cmd_spec(format))`
- [x] **事後確認**: `cargo check --bin fav` — エラー 0
- [x] **後方互換確認**: `cargo test v240000 --bin fav` — 5/5 PASS（version_is_24_0_0 削除前の全件）

---

### T3: `fav/src/driver.rs` — `v241000_tests` 追加

- [x] **事前確認**: `grep -n "fn version_is_24_0_0" fav/src/driver.rs | head -3`
- [x] **T3-1（T4-1 より前に必須）**: `v240000_tests::version_is_24_0_0` テスト関数を**削除**
- [x] **T3-2**: `v241000_tests` モジュールを `v240000_tests` の直後に追加（5 件）
  - `version_is_24_1_0`
  - `cmd_spec_markdown_has_type_system`
  - `cmd_spec_markdown_has_opcodes`
  - `cmd_spec_html_has_h1`
  - `changelog_has_v24_1_0`
- [x] `cargo test v241000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1934 件合格）を確認

---

### T4: Cargo.toml + CHANGELOG + README + benchmarks + spec.mdx

- [x] `fav/Cargo.toml` の `version = "24.0.0"` → `"24.1.0"` に変更
- [x] `CHANGELOG.md` 先頭に v24.1.0 エントリを追加
- [x] `README.md` の `fav doc` コマンド行の直後に `fav spec --format markdown > SPEC.md` を追加
- [x] `benchmarks/v24.1.0.json` を新規作成（test_count: 1934）
- [x] `site/content/docs/tools/spec.mdx` を新規作成（4 セクション説明・オプション・既知制限）
- [x] `cargo test v241000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1934 件合格）

---

## テスト一覧（v241000_tests、5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_1_0` | Cargo.toml に `version = "24.1.0"` | — |
| `cmd_spec_markdown_has_type_system` | `cmd_spec("markdown")` に `"型システム"` | — |
| `cmd_spec_markdown_has_opcodes` | `cmd_spec("markdown")` に `"0x01"`（Const opcode） | — |
| `cmd_spec_html_has_h1` | `cmd_spec("html")` に `"<h1>"`（md_to_html 変換確認） | — |
| `changelog_has_v24_1_0` | CHANGELOG.md に `[v24.1.0]` | — |

---

## 完了条件チェックリスト

- [x] `fn build_spec_markdown()` が実装される（4 セクション含む）
- [x] `pub fn cmd_spec(format: &str) -> String` が実装される
- [x] `main.rs` に `"spec"` サブコマンドが追加される
- [x] `v240000_tests::version_is_24_0_0` が削除済み（T4-1 より前）
- [x] `cargo test v241000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1934 件合格）
- [x] `CHANGELOG.md` に v24.1.0 エントリ
- [x] `benchmarks/v24.1.0.json` 作成済み（test_count: 1934）
- [x] `README.md` に `fav spec` セクション追加済み
- [x] `site/content/docs/tools/spec.mdx` 作成済み

---

## コードレビュー対応（2026-06-23 — spec-reviewer 指摘）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [HIGH] | tasks.md T1-1 の挿入位置「直前」→「直後」 | 修正済み（実装も直後に配置） |
| [HIGH] | plan.md の注意書きとコードが矛盾 | plan.md 注意書きを実態に合わせて修正 |
| [MED] | `opcode_count: 30` → `31` | plan.md / benchmarks JSON 修正済み |
| [MED] | `spec.mdx` がスコープ外 | spec.md / plan.md / tasks.md に追加、T4 で実装済み |

## コードレビュー対応（2026-06-23 — code-reviewer 指摘）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [MED] | opcode 表の無言省略（31 件中 20 件のみ掲載） | `### 2.1 opcode 一覧（主要命令）` に改め、省略対象（ChainCheck / JumpIfNotVariant / スーパー命令等）を明記 |
| [MED] | 未知 format のサイレント fallback | `"markdown" \| other =>` に分岐し、unknown 値には `eprintln!("warning: ...")` を出力してから markdown を返すよう修正 |
| [LOW] | Section 4.1 誤字「网羅性条件」→「網羅性条件」 | 修正済み |
| [LOW] | フォールバックテスト欠如 | スキップ（次バージョン検討） |
| [LOW] | HTML テーブル制限の非テスト | スキップ（制限は spec.mdx に明記済み） |
| [MED] | HTML テーブルの既知制限が未明記 | spec.md「既知の制限」セクション + spec.mdx に明記 |
| [LOW] | plan.md `push_str` 言及の揺れ | [HIGH] 修正に統合 |

## 実装時の注意事項（実績）

| # | 内容 | 対応方針 |
|---|---|---|
| 1 | `SPEC_CONTENT` の backtick コードフェンスが raw string `r#"..."#` を閉じるか | 閉じない。backtick は `#` ではないため `"#` のデリミタに影響しない |
| 2 | `format!("{}{}", header, SPEC_CONTENT)` で SPEC_CONTENT の `{` `}` が干渉するか | 干渉しない。SPEC_CONTENT は第 2 位置引数として渡されるため、内部の `{` `}` はフォーマット文字列として再解釈されない |
| 3 | `cmd_spec("html")` の `md_to_html` が `<h1>` を生成するか | 生成する。`# Favnir 言語仕様書` が `<h1>Favnir 言語仕様書</h1>` に変換される（line 10267: `line.strip_prefix("# ")` → `format!("<h1>{}</h1>\n", ...)` ）|
