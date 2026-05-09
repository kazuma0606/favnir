# Favnir v2.0.0 仕様書 — キーワードリネーム完全移行 + セルフホスト・マイルストーン

作成日: 2026-05-09

> **テーマ**: v1.x で予告してきた破壊的変更（`trf`/`flw`/`cap` 削除）を一括適用し、
> `stage`/`seq`/`interface` のみの新構文に移行する。
> `fav migrate` による自動移行ツールを同時提供し、既存コードの移行コストを最小化する。
> セルフホスト・マイルストーンとして Favnir 製レキサーを VM 上で動かす。
>
> **前提**: v1.9.0 完了（523 テスト通過）

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 | `v2.0.0` がビルドされ HELP テキストに反映される |
| 1 | 旧キーワード削除 | `trf`/`flw`/`cap` がコンパイルエラーになる（移行ヒント付き） |
| 2 | `abstract stage`/`abstract seq` | `abstract trf`→`abstract stage`、`abstract flw`→`abstract seq` に完全移行 |
| 3 | `fav migrate` コマンド | v1.x コードを v2.0.0 構文に自動変換する |
| 4 | セルフホスト・マイルストーン | Favnir で書いたレキサーが VM 上で動く |
| 5 | テスト・ドキュメント | 全テスト通過、langspec v2.0.0、migration-guide.md |

---

## 2. Phase 0 — バージョン更新

- `Cargo.toml`: `version = "2.0.0"`
- `main.rs`: HELP テキスト `v2.0.0`
- `src/backend/artifact.rs`: FVC magic byte/version を `v0x20` に更新

---

## 3. Phase 1 — 旧キーワード削除

### 3-1. 設計方針

v1.9.0 では `stage`/`seq` を `trf`/`flw` のエイリアスとして共存させた。
v2.0.0 では **旧キーワードをコンパイルエラー**にする。

エラーメッセージには移行先を明示し、`fav migrate` との連携を案内する。

```
error[E2001]: keyword `trf` has been removed in v2.0.0
  --> main.fav:3:1
  |
3 | trf double: Int -> Int = |x| x * 2
  | ^^^ use `stage` instead (run `fav migrate` to auto-fix)
```

### 3-2. 削除対象キーワード

| 旧キーワード | 削除理由 | 移行先 | エラーコード |
|---|---|---|---|
| `trf` | `stage` に統一 | `stage` | E2001 |
| `flw` | `seq` に統一 | `seq` | E2002 |
| `cap` | `interface` に統一 | `interface` | E2003 |
| `abstract trf` | `abstract stage` に統一 | `abstract stage` | E2001 |
| `abstract flw` | `abstract seq` に統一 | `abstract seq` | E2002 |

### 3-3. 実装方針

**字句解析 (`src/frontend/lexer.rs`)**:
- `"trf"` / `"flw"` / `"cap"` のキーワードマッピングを**保持**する
  （トークン認識は維持し、パーサー側でエラーを出す）
- これにより "unexpected token" ではなく親切なエラーメッセージが出せる

**パーサー (`src/frontend/parser.rs`)**:
- `parse_item` / `parse_abstract_item` の `TokenKind::Trf` / `TokenKind::Flw` / `TokenKind::Cap` の分岐を、
  それぞれ `E2001` / `E2002` / `E2003` を返す ParseError に置き換える
- `expect_any([TokenKind::Trf, TokenKind::Stage])` → `expect_any([TokenKind::Stage])` に縮小
- `expect_any([TokenKind::Flw, TokenKind::Seq])` → `expect_any([TokenKind::Seq])` に縮小

**テスト更新**:
- `trf`/`flw`/`cap` を使っている全テスト・example を `stage`/`seq`/`interface` に書き換える

### 3-4. `cap` の削除詳細

`cap` は v0.4.0 で導入され、v1.1.0 で `interface` に置き換えられた。
v2.0.0 では `cap` の構文（`cap Name<T> = { ... }`）をパーサーレベルで削除する。

- `parse_cap_def` を削除（または E2003 を返す stub に置き換え）
- `ast.rs` の `Item::CapDef(CapDef)` は将来の互換性のため残す（しかし生成されなくなる）
- checker.rs / compiler.rs の CapDef ハンドラは残したままでよい（到達しなくなる）

### 3-5. エラーコード

| コード | 条件 |
|---|---|
| E2001 | `trf` または `abstract trf` キーワードを使用 |
| E2002 | `flw` または `abstract flw` キーワードを使用 |
| E2003 | `cap` キーワードを使用 |

> **注**: これらは **Parse エラー**（型検査エラーではない）。
> 既存の E001-E069 系とは別カテゴリ（E2xxx = v2.0.0 移行エラー）として扱う。

---

## 4. Phase 2 — `abstract stage` / `abstract seq`

### 4-1. 変更内容

v1.x では `abstract trf`/`abstract flw` として定義されていたパイプライン抽象を、
`abstract stage`/`abstract seq` に完全移行する。

**パーサー変更**:

```rust
// parse_abstract_item 内
// Before:
TokenKind::Trf | TokenKind::Stage => Ok(Item::AbstractTrfDef(...))
TokenKind::Flw | TokenKind::Seq   => Ok(Item::AbstractFlwDef(...))

// After:
TokenKind::Stage => Ok(Item::AbstractTrfDef(...))  // TrfはE2001エラー
TokenKind::Seq   => Ok(Item::AbstractFlwDef(...))  // FlwはE2002エラー
```

### 4-2. AST 変更なし

`Item::AbstractTrfDef` / `Item::AbstractFlwDef` は AST 上の名称として維持する。
（内部名は Rust 実装の都合；ユーザーに見える名前は `abstract stage`/`abstract seq`）

### 4-3. Example ファイル更新

- `examples/abstract_flw_basic.fav` → `abstract seq` 記法に更新
- `examples/abstract_flw_inject.fav` → `abstract seq` 記法に更新

---

## 5. Phase 3 — `fav migrate` コマンド

### 5-1. 設計方針

`fav migrate` は v1.x 構文のファイルを v2.0.0 構文に自動変換するテキスト変換ツール。
AST パースは行わず、**正規表現ベースのトークン置換**で実装する。

これにより `trf`/`flw`/`cap` を含む既存コードが `fav check` を通るまでを1コマンドで達成できる。

### 5-2. 変換規則

| 対象パターン | 変換後 | 注意 |
|---|---|---|
| `trf ` (行頭またはスペース後) | `stage ` | 識別子内の `trf` は変換しない |
| `flw ` | `seq ` | |
| `abstract trf ` | `abstract stage ` | |
| `abstract flw ` | `abstract seq ` | |
| `cap ` (定義) | `// TODO: migrate cap to interface\n` + 元の行 | 自動変換不可フラグ |
| `public trf ` | `public stage ` | visibility 修飾子と組み合わせ |
| `public flw ` | `public seq ` | |
| `private trf ` | `private stage ` | |
| `private flw ` | `private seq ` | |

**注**: `cap` は `interface` と意味的に異なる部分があるため、完全自動変換はせず
`// TODO:` コメントで手動確認を促す。

### 5-3. CLI 仕様

```
fav migrate [OPTIONS] <file|dir>

OPTIONS:
  --in-place          ファイルを直接書き換える（デフォルト: stdout に出力）
  --dry-run           変換内容を表示するだけで書き換えない
  --dir <path>        ディレクトリ内の全 .fav ファイルを再帰的に処理
  --check             変換が必要なファイルがあれば exit 1（CI 用）
```

**出力例** (`--dry-run`):

```
Would migrate main.fav:
  3:  trf double: Int -> Int = |x| x * 2
  3: stage double: Int -> Int = |x| x * 2

  7:  flw pipeline = double |> normalize
  7: seq pipeline = double |> normalize

2 replacements in 1 file.
```

### 5-4. 実装場所

- `src/driver.rs`: `cmd_migrate(path, in_place, dry_run, check_mode)` を追加
- `main.rs`: `migrate` サブコマンドを追加

```rust
// main.rs HELP 追加
//   migrate  [--in-place] [--dry-run] <file|dir>
//            migrate v1.x code to v2.0.0 syntax
```

### 5-5. 移行できないパターン（手動対応が必要）

| パターン | 理由 |
|---|---|
| `cap Name<T> = { ... }` | `interface` との意味の差がある |
| 識別子名が `trf`/`flw` と衝突する場合 | コンテキスト判断が困難 |
| マクロ・文字列内の `trf` テキスト | テキスト置換の誤爆リスク |

---

## 6. Phase 4 — セルフホスト・マイルストーン

### 6-1. 目標

Favnir 製のレキサー（字句解析器）を Favnir で書き、
Rust VM 上で動かすことで「Favnir で Favnir を処理する」最初のマイルストーンを達成する。

v2.0.0 では**完全なセルフホストは目指さない**。
パーサー・型チェッカーのセルフホストは v2.1.0 以降に持ち越す。

### 6-2. Favnir 製レキサーの仕様

対象: Favnir の数値リテラル・識別子・演算子を認識するサブセットレキサー

```favnir
// examples/selfhost/lexer.fav
type Token = Int_Lit(Int) | Ident(String) | Plus | Minus | Eof

stage tokenize_int: String -> Token = |s| {
    bind n <- String.to_int(s)
    n ?? ...  // to be continued
}
```

**認識できるトークン**:
- 整数リテラル (`123`, `-42`)
- 識別子 (`foo`, `snake_case`)
- 演算子 (`+`, `-`, `*`, `/`, `==`, `!=`)
- キーワード (`fn`, `stage`, `seq`, `bind`, `if`, `else`)
- EOF

### 6-3. 実装対象ファイル

- `examples/selfhost/lexer.fav` — メインレキサー実装
- `examples/selfhost/lexer.test.fav` — レキサーのテスト
- `fav test examples/selfhost/lexer.test.fav` で全テスト通過が確認条件

### 6-4. 技術的な前提条件

Favnir 製レキサーには以下の標準ライブラリ関数が必要:
- `String.char_at(s, i)` — i 番目の文字を返す（既存）
- `String.length(s)` — 文字列長（既存または追加）
- `String.slice(s, start, end)` — 部分文字列（既存）
- `String.to_int(s)` — 数値変換（既存）
- `List.range(0, n)` — インデックスイテレーション（既存）

必要に応じて不足する標準ライブラリ関数を `vm.rs` に追加する。

### 6-5. `fav explain compiler` コマンド

```
fav explain compiler [file]
```

コンパイル工程の可視化:
- Phase 1: Lexer → Token 数
- Phase 2: Parser → AST ノード数
- Phase 3: Checker → 型付き AST、エラー数
- Phase 4: Compiler → IR 関数数、グローバル数
- Phase 5: Codegen → バイトコードサイズ
- Phase 6: (selfhost) → Favnir 製レキサー使用時の工程

---

## 7. Phase 5 — テスト・ドキュメント

### 7-1. テスト要件

#### 旧キーワードエラー

| テスト名 | 検証内容 |
|---|---|
| `trf_keyword_removed_e2001` | `trf F: Int -> Int = \|x\| x` で E2001 |
| `flw_keyword_removed_e2002` | `flw P = F` で E2002 |
| `cap_keyword_removed_e2003` | `cap Eq<T> = { ... }` で E2003 |
| `abstract_trf_removed_e2001` | `abstract trf F: Int -> Int` で E2001 |

#### `fav migrate`

| テスト名 | 検証内容 |
|---|---|
| `migrate_trf_to_stage` | `trf F: Int -> Int = \|x\| x` が `stage F: ...` に変換される |
| `migrate_flw_to_seq` | `flw P = F` が `seq P = F` に変換される |
| `migrate_abstract_trf` | `abstract trf F: Int -> Int` → `abstract stage F: Int -> Int` |
| `migrate_no_false_positive` | 識別子 `trf_count` が変換されない |
| `migrate_public_visibility` | `public trf` → `public stage` |

#### セルフホスト・レキサー

| テスト名 | 検証内容 |
|---|---|
| `selfhost_lexer_tokenizes_ints` | `"123"` が `Int_Lit(123)` を返す |
| `selfhost_lexer_tokenizes_ident` | `"foo"` が `Ident("foo")` を返す |
| `selfhost_lexer_tokenizes_keyword` | `"stage"` がキーワードトークンを返す |
| `selfhost_lexer_eof` | 空文字列が `Eof` を返す |

### 7-2. ドキュメント

- `versions/v2.0.0/langspec.md` — v2.0.0 言語仕様書（全面改訂）
- `versions/v2.0.0/migration-guide.md` — v1.x → v2.0.0 移行ガイド
- `RELEASE_NOTES.md` — v2.0.0 リリースノート追記

---

## 8. エラーコード一覧（v2.0.0 追加分）

| コード | Phase | 条件 |
|---|---|---|
| E2001 | 1 | `trf` または `abstract trf` キーワードを使用（v2.0.0 で削除）|
| E2002 | 1 | `flw` または `abstract flw` キーワードを使用（v2.0.0 で削除） |
| E2003 | 1 | `cap` キーワードを使用（v2.0.0 で削除） |

> **将来計画**: 既存 E001-E069 の 4 桁化（E0001-E0069）は v2.1.0 で実施。
> セルフホスト・チェッカー向けの E1xxx-E9xxx 体系はその際に確定する。

---

## 9. 後方互換性

v2.0.0 は **メジャーバージョン** であり、v1.x との後方互換を**意図的に破る**。

| 変更 | 影響 | 対策 |
|---|---|---|
| `trf` 削除 | `trf` を使う全 .fav ファイルがエラー | `fav migrate` |
| `flw` 削除 | `flw` を使う全 .fav ファイルがエラー | `fav migrate` |
| `cap` 削除 | `cap` を使う全 .fav ファイルがエラー | 手動移行（`// TODO:` コメント案内） |
| `.fvc` magic byte 変更 | v1.x の `.fvc` は v2.0.0 で実行不可 | 再コンパイル必要 |

### 後方互換を維持するもの

- `stage`/`seq`/`interface`/`impl`/`fn`/`bind`/`chain`/`yield`/`collect`/`match`/`if`/`for`/`??` は変更なし
- 標準ライブラリ API (`List.*`, `String.*`, `Map.*`, etc.) は変更なし
- `fav run`/`fav check`/`fav test`/`fav fmt`/`fav lint`/`fav bench` の動作は変更なし

---

## 10. 完了条件（Done Definition）

- [ ] `trf F: Int -> Int = |x| x` が E2001 を発してコンパイルエラーになる
- [ ] `flw P = F` が E2002 を発してコンパイルエラーになる
- [ ] `cap Eq<T> = { ... }` が E2003 を発してコンパイルエラーになる
- [ ] `abstract trf F: Int -> Int` が E2001 を発する
- [ ] `stage`/`seq`/`interface` で書かれたコードが正常にコンパイル・実行される
- [ ] `fav migrate trf_example.fav` が `stage` 記法に変換する
- [ ] `fav migrate flw_example.fav --in-place` がファイルを書き換える
- [ ] `examples/selfhost/lexer.fav` が `fav run` で動く
- [ ] `examples/selfhost/lexer.test.fav` の全テストが `fav test` で通る
- [ ] v1.9.0 の全テスト（523）から `trf`/`flw`/`cap` 使用テストを `stage`/`seq`/`interface` に書き換えた上で全て通る
- [ ] `cargo build` で警告ゼロ
- [ ] `Cargo.toml` バージョンが `"2.0.0"`

---

## 11. 先送り一覧（v2.0.0 では対応しない）

| 機能 | 理由 | 対応予定 |
|---|---|---|
| エラーコード 4 桁化 (E0001-E0999) | 大規模な全テスト更新が必要 | v2.1.0 |
| 型チェッカーの Favnir 移植 | レキサー移植が前提 | v2.1.0+ |
| パーサーの Favnir 移植 | 型チェッカー移植が前提 | v2.2.0+ |
| tokio 実ランタイム統合 | 大規模依存追加 | 検討中 |
| `collect` 内の `for`+`yield` | 設計未確定 | v2.1.0 |
| `fav explain compiler` の完全実装 | 優先度中 | v2.1.0 |
