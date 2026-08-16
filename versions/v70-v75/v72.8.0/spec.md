# v72.8.0 Spec — インタラクティブチュートリアル（`fav learn`）

Date: 2026-08-12
Status: 計画中

---

## 背景

`fav learn` コマンドを新設し、Favnir 入門者が対話的にパイプライン構文を学べる
チュートリアルを提供する。5 章構成で、各章はプロンプト・期待入力・ヒントを持つ。

---

## 目標

```bash
$ fav learn
Favnir インタラクティブチュートリアル v1.0

Chapter 1: 最初のパイプライン
[1/5] fn main(ctx: AppCtx) -> Result<Unit, String> を書いてみましょう
>>> fn main(ctx: AppCtx) -> Result<Unit, String> { Ok(()) }
✓ 正解！ 次へ進みます。

Chapter 2: 型システムの力
[2/5] schema を使ってフィールドを定義してみましょう
>>> schema Row { id: String }
✓ 正解！

...

Chapter 5: 分散実行
[5/5] par を使って並列ステージを書いてみましょう
>>> par [LoadA, LoadB]
✓ 全章完了！ fav.dev/docs で次のステップへ。
```

---

## 実装詳細

### `driver.rs` — `LearnChapter` 構造体

```rust
pub struct LearnChapter {
    pub chapter: u32,
    pub title: &'static str,
    pub prompt: &'static str,
    pub hint: &'static str,
    pub expected_contains: &'static str,
}
```

フィールド:
- `chapter` — 章番号（1〜5）
- `title` — 章タイトル
- `prompt` — ユーザーへの課題文
- `hint` — 不正解時に表示するヒント
- `expected_contains` — ユーザー入力が含むべきキーワード（部分一致で正解判定）

### `driver.rs` — `LEARN_CHAPTERS: &[LearnChapter]`

5 章分の静的データ:

| 章 | タイトル | expected_contains |
|---|---|---|
| 1 | 最初のパイプライン | `fn main` |
| 2 | 型システムの力 | `schema` |
| 3 | Rune を使ったデータ処理 | `import rune` |
| 4 | AI パイプライン | `Llm`（大文字固定・部分一致）|
| 5 | 分散実行 | `par` |

### `driver.rs` — `cmd_learn()`

```rust
pub fn cmd_learn()
```

- 各章を順番に表示（章番号・タイトル・プロンプト）
- stdin から 1 行読み込み
- `expected_contains` を含む場合: `✓ 正解！ 次へ進みます。` を表示して次章へ
- 含まない場合: `ヒント: {hint}` を表示して再入力を促す
- 全 5 章クリア後: `全章完了！ fav.dev/docs で次のステップへ。` を表示

進捗保存（`~/.fav_learn_progress`）はスコープ外（後述）。

### `main.rs` — `fav learn` コマンド追加

```rust
Some("learn") => {
    crate::driver::cmd_learn();
}
```

---

## テスト

### `v728000_tests` モジュール

`cmd_learn` は stdin 依存のためユニットテスト困難。
`LEARN_CHAPTERS` の静的データ存在を検証する：

```rust
#[cfg(test)]
mod v728000_tests {
    use super::LEARN_CHAPTERS;

    #[test]
    fn learn_chapter1_exists() {
        assert!(LEARN_CHAPTERS.len() >= 1,
            "LEARN_CHAPTERS should have at least 1 entry");
        assert_eq!(LEARN_CHAPTERS[0].chapter, 1,
            "first chapter should be chapter 1");
        assert!(!LEARN_CHAPTERS[0].title.is_empty(),
            "chapter 1 title should not be empty");
        assert!(
            LEARN_CHAPTERS[0].expected_contains.contains("fn") ||
            LEARN_CHAPTERS[0].expected_contains.contains("main"),
            "chapter 1 should test pipeline entry point"
        );
    }

    #[test]
    fn learn_chapter5_exists() {
        assert!(LEARN_CHAPTERS.len() >= 5,
            "LEARN_CHAPTERS should have at least 5 entries");
        assert_eq!(LEARN_CHAPTERS[4].chapter, 5,
            "fifth chapter should be chapter 5");
        assert!(!LEARN_CHAPTERS[4].title.is_empty(),
            "chapter 5 title should not be empty");
        assert!(
            LEARN_CHAPTERS[4].expected_contains.contains("par"),
            "chapter 5 should test distributed/par construct"
        );
    }
}
```

---

## 成功基準

- `cargo test v728000` で 2 件 pass
- `cargo test` 全体で 3640 tests pass（3638 + 2）
- `fav/Cargo.toml` のバージョンが `72.8.0` であること
- `LEARN_CHAPTERS` に 5 エントリ存在
- `cmd_learn` が `pub fn` で存在
- `main.rs` で `fav learn` が `cmd_learn()` を呼ぶ

---

## スコープ外

- 進捗保存（`~/.fav_learn_progress`）— 複雑度のため v73.x 以降
- `:skip` / `:quit` コマンド — v73.x 以降
- サイト側ドキュメント更新（v73.x 以降）
- stdin/stdout テスト（ファイルシステム・プロセス依存のため除外）

---

## 変更ファイル

- `fav/src/driver.rs` — `LearnChapter` 構造体 + `LEARN_CHAPTERS` 静的データ + `cmd_learn` + `v728000_tests` + バージョン更新
- `fav/src/main.rs` — `fav learn` コマンド追加（`cmd_learn` import + `Some("learn")` アーム）
- `fav/Cargo.toml` — version `72.7.0` → `72.8.0`
- `CHANGELOG.md` — v72.8.0 エントリ追加
- `versions/current.md` — 進行中バージョンを v72.8.0 に更新
