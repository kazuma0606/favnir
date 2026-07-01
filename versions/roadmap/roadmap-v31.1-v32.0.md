# Roadmap v31.1.0 〜 v32.0.0 — Language Polish

Date: 2026-07-01

## 目標

v31.0「Real-World Readiness」で「動く」を確認した。
次の問いは **「書いていて気持ちいいか」** だ。

機能は揃っている。しかし「初めて使うエンジニアが 30 分で詰まらずに動かせるか」という
体験品質はまだ不十分だ。特に 3 点が鍵を握る:

1. **エラーメッセージ** — 「何が悪いか」より「どう直すか」を教える
2. **REPL** — データ探索ツールとして使えるか
3. **LSP** — エディタとの統合品質

> **Language Polish の定義（本プロジェクト固有）**
> 「Favnir を初めて使うデータエンジニアが、エラーメッセージを見て
>  自力でコードを修正し、30 分以内に最初のパイプラインを動かせること」

**完了条件（最終テスト）:**

```bash
# 1. 全 Rust テスト通過
cargo test

# 2. エラーメッセージ品質確認
# （未定義変数を書いたとき hint が表示される）
echo 'fn main() -> Int { foo + 1 }' | fav check /dev/stdin
# → [E0001] hint: `foo` は未定義です。 ... と typo 候補が出る

# 3. fav explain
fav explain E0001

# 4. REPL コマンド
echo ":doc List.group_by" | fav repl

# 5. fav test --watch（手動確認）
fav test --watch src/

# 6. fav check --all（プロジェクト全体）
fav check --all
```

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| エラー表示形式 | rustc スタイル（`-->` ファイル位置 + `|` ソース行 + `= ヒント:`）|
| typo 候補 | Levenshtein 距離 ≤ 2 のシンボルを最大 3 件提示 |
| エラーコード URL | `https://favnir.dev/errors/E0001` 形式（全エラーコードに付与）|
| `fav explain` | エラーコード → 説明・例・修正方法を stdout に出力 |
| REPL タブ補完 | rustyline 等は使わず、独自のシンプルな補完実装 |
| LSP Inlay Hints | `textDocument/inlayHint`（型推論結果を `bind x <- expr` の右に表示）|
| `fav test --watch` | 500ms ポーリング（`fav watch` と同じ実装を流用）|
| `fav check --all` | `fav.toml` の `src` 以下全 .fav をスキャンしてクロスファイル型エラーを報告 |
| `fav scaffold` | `fav scaffold stage MyStage` で既存プロジェクトに stage を追加 |
| 破壊的変更 | なし |

---

## バージョン計画

### v31.1 — エラーメッセージ v2（rustc スタイル）

**テーマ**: エラーを見て「すぐ直せる」と感じるメッセージにする。

**現状**:
```
[E0001] undefined variable: user_id
```

**目標**:
```
error[E0001]: undefined variable: user_id
  --> src/stages.fav:12:5
   |
12 |   transform(user_id, name)
   |             ^^^^^^^ この変数は未定義です
   |
   = ヒント: `userId` (line 8) で定義された変数の typo ではないですか？
   = 参照: https://favnir.dev/errors/E0001
```

**実装内容**:
- `format_diagnostic()` 関数を rustc スタイルに刷新
- `span` 情報（行番号・列番号・ソース行テキスト）をすべてのエラーに付与
- `hint:` / `note:` / `help:` フィールドを `CheckError` / `LintError` 構造体に追加
- 主要エラーコード（E0001〜E0021）に `hint` テキストを設定

完了条件:
- E0001〜E0010 の全エラーに `hint:` が付与されている
- ソース行とカーソル位置（`^^^^^`）が表示される
- エラーコード URL が末尾に表示される
- Rust テスト 2 件追加

---

### v31.2 — typo 候補（Levenshtein）+ エラーコード全件 URL

**テーマ**: 変数名・関数名・型名の typo を自動検出して候補を提示する。

**実装**:
```rust
// Levenshtein 距離で候補を検索
fn suggest_similar(name: &str, candidates: &[&str]) -> Vec<&str> {
    candidates.iter()
        .filter(|c| levenshtein(name, c) <= 2)
        .take(3)
        .collect()
}
```

適用箇所:
- `E0001`（未定義変数）→ スコープ内の変数名から候補
- `E0007`（未定義関数）→ 定義済み関数名から候補
- `E0011`（未定義型）→ 定義済み型名から候補

また、E0001〜E0320 全エラーコードに `https://favnir.dev/errors/Exxxx` URL を付与する。

完了条件:
- typo を書いたとき `= ヒント: 'foo_id'（line 8）の typo ではないですか？` が表示される
- 全エラーコードに URL が付与されている
- Rust テスト 2 件追加

---

### v31.3 — fav explain E0001 コマンド

**テーマ**: エラーコードの詳細説明をコマンドラインで参照できるようにする。

```bash
$ fav explain E0001
error[E0001]: undefined variable

説明:
  スコープ内に定義されていない変数を参照しようとしました。

よくある原因:
  1. 変数名の typo（例: `userId` を `user_id` と書いた）
  2. bind の前に変数を使用した
  3. 別の関数スコープで定義された変数を参照した

修正例:
  // NG
  fn process() -> String {
      user_id  // 未定義
  }

  // OK
  fn process(user_id: String) -> String {
      user_id
  }

参照: https://favnir.dev/errors/E0001
```

E0001〜E0021 の全エラーコードに説明・原因・修正例を実装する。

完了条件:
- `fav explain E0001` 〜 `fav explain E0021` が説明を出力する
- `fav explain unknown` が利用可能なエラーコード一覧を出力する
- Rust テスト 1 件追加

---

### v31.4 — REPL 品質向上（:doc / :load / :history / タブ補完）

**テーマ**: `fav repl` をデータ探索ツールとして実用レベルにする。

**追加コマンド**:

```
favnir> :doc List.group_by
  List.group_by : List<T> -> (T -> String) -> Map<String, List<T>>

  リストをキー関数でグループ化する。

  例:
    bind grouped <- List.group_by(rows, |r| r.category)
    Map.get(grouped, "active")

favnir> :load src/pipeline.fav
  loaded: LoadCsv, ValidateRows, WriteToDb, ...

favnir> :history
  1: List.length([1, 2, 3])
  2: String.split("a,b,c", ",")
  ...

favnir> :save session.fav
  saved to session.fav

favnir> List.g<Tab>
  List.get    List.group_by
```

**実装内容**:
- `:doc <fn>` — stdlib + 定義済み関数のドキュメントを表示
- `:load <file>` — .fav ファイルを REPL 環境にロード
- `:history` — 入力履歴を表示（最大 100 件）
- `:save <file>` — セッション定義をファイルに保存
- タブ補完 — モジュール名 + 関数名の補完（`:` コマンドも対象）

完了条件:
- 上記 5 コマンドが動作する
- タブ補完が `List.` + `<Tab>` で関数一覧を返す
- Rust テスト 1 件追加

---

### v31.5 — LSP Inlay Hints（型推論結果インライン表示）

**テーマ**: `bind x <- expr` の型推論結果をエディタでインライン表示する。

```favnir
bind rows <- LoadCsv(path)   // : List<RawRow>  ← インライン表示
bind n    <- List.length(rows) // : Int
```

**実装内容**:
- LSP `textDocument/inlayHint` リクエストに対応
- `bind` ステートメントの変数に推論型を `// : Type` 形式でヒント表示
- 関数の戻り型推論結果を表示（明示的型注釈がない場合）
- VS Code 拡張の `package.json` に `inlayHints` 機能を追記

完了条件:
- `lsp/mod.rs` に `handle_inlay_hints` が実装されている
- VS Code で bind の右に型が表示される（手動確認）
- Rust テスト 1 件追加

---

### v31.6 — fav test --watch

**テーマ**: ファイル変更を検知してテストを自動再実行する。

```bash
$ fav test --watch src/
[watch] テストを監視中... (Ctrl+C で終了)
[12:34:01] 変更検知: src/validators.fav
[12:34:01] テスト実行中...
[12:34:02] PASS: validate_row_ok (0.3ms)
[12:34:02] PASS: validate_row_missing_field (0.1ms)
[12:34:02] 2/2 テスト通過
```

**実装内容**:
- `fav watch` の実装（500ms ポーリング）を `fav test --watch` でも使えるように
- ファイル変更時に `fav test <changed_files>` を自動実行
- Ctrl+C でクリーンに終了

完了条件:
- `fav test --watch` でファイル変更を検知してテストが再実行される
- Rust テスト 1 件追加

---

### v31.7 — fav check --all（クロスファイル型エラー）

**テーマ**: プロジェクト内の全ファイルを一括型チェックして問題を発見する。

**現状**: `fav check src/main.fav` は 1 ファイルしかチェックしない（import は解決されるが）。

**目標**:
```bash
$ fav check --all
checking src/types.fav... ok
checking src/validators.fav... ok
checking src/stages.fav...
  error[E0009] src/stages.fav:34:5 — 型不一致: Int が必要ですが String が返っています
checking src/main.fav... ok

1 エラーが見つかりました
```

**実装内容**:
- `fav.toml` の `src` ディレクトリ以下の全 .fav をスキャン
- 各ファイルを独立してチェック（import 解決込み）
- 全エラーをまとめて報告
- `--json` フラグで JSON 形式出力（LSP 連携向け）

完了条件:
- `fav check --all` でプロジェクト全体をチェックできる
- `--json` で JSON 出力される
- Rust テスト 1 件追加

---

### v31.8 — fav scaffold（既存プロジェクトへのコード追加）

**テーマ**: 既存プロジェクトに新しいステージ・シーケンス・Rune を追加するコマンド。

```bash
# 新しい stage を src/stages.fav に追加
fav scaffold stage EnrichRows

# 新しい seq パイプラインを追加
fav scaffold seq AuditPipeline

# Rune を fav.toml に追加（fav add の alias）
fav scaffold rune kafka
```

**生成コード（stage の例）**:
```favnir
// generated by fav scaffold stage EnrichRows
stage EnrichRows: List<ValidRow> -> List<EnrichedRow> !IO = |rows| {
    // TODO: implement
    Result.ok(rows |> List.map(|row| row))
}
```

完了条件:
- `fav scaffold stage <Name>` が src/ に stub コードを追加する
- `fav scaffold seq <Name>` が seq 定義を追加する
- Rust テスト 1 件追加

---

### v31.9 — ドッグフード修正 vol.2

**テーマ**: v31.1〜v31.8 の実装中・ドッグフードで発見した問題を修正する。

想定される修正候補（v31.0 完了時点で確定）:
- エラーメッセージ改善の残課題
- REPL の edge case 修正
- LSP の安定性改善
- `fav check --all` のパフォーマンス最適化

完了条件:
- 既知の残課題が修正済み
- テスト全件通過

---

## v32.0 — Language Polish マイルストーン宣言

**完了条件:**

| コンポーネント | 完了基準 |
|---|---|
| エラーメッセージ v2 | E0001〜E0021 全件に `hint:` / `note:` 付与、rustc スタイル表示 |
| typo 候補 | 変数/関数/型の typo 時に候補提示（Levenshtein ≤ 2）|
| エラーコード URL | 全コードに `https://favnir.dev/errors/Exxxx` 付与 |
| fav explain | E0001〜E0021 の説明・修正例が表示される |
| REPL | :doc / :load / :history / :save / タブ補完 が動作 |
| LSP Inlay Hints | bind 変数の型がエディタでインライン表示される |
| fav test --watch | ファイル変更で自動テスト再実行 |
| fav check --all | プロジェクト全体クロスファイルチェック |
| fav scaffold | stage / seq / rune の追加コマンドが動作 |

**最終テスト:**

```bash
cargo test
fav explain E0001
fav explain E0021
fav check --all
fav test --watch src/ &  # バックグラウンドで起動確認後 Ctrl+C
```

**★ クリーンアップ実施:**

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
cargo build
cargo test 2>&1 | grep "test result"
du -sh target/
```

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v30.1-v35.0.md`
- 前フェーズ: `versions/roadmap/roadmap-v30.1-v31.0.md`
- 次フェーズ: `versions/roadmap/roadmap-v32.1-v33.0.md`
