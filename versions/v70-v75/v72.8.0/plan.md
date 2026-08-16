# v72.8.0 実装計画 — インタラクティブチュートリアル（`fav learn`）

Date: 2026-08-12

---

## 依存順序

```
T0: 事前確認
  ↓
T1: LearnChapter 構造体 + LEARN_CHAPTERS 静的データ追加（driver.rs）
  ↓
T2: cmd_learn 追加（driver.rs）
  ↓
T3: main.rs — fav learn コマンド追加
  ↓
T4: v728000_tests モジュール追加（driver.rs）
  ↓
T5: Cargo.toml バージョン更新 + driver.rs バージョンアサーション更新
  ↓
T6: 部分テスト確認（cargo test v728000）
  ↓
T7: 全体テスト確認（cargo test）
  ↓
T8: CHANGELOG.md 更新
T9: versions/current.md 更新
  ↓
T10: 最終確認
```

---

## ステップ詳細

### T0: 事前確認

- `fav/Cargo.toml` のバージョンが `72.7.0` であることを確認
- `cargo test` が 3638 tests pass（0 failures）であることを確認
- `driver.rs` に `v727000_tests` モジュールが存在することを確認
- `driver.rs` に `v728000_tests` が未存在であることを確認
- `driver.rs` 内の `"72.7.0"` バージョンアサーション文字列の件数を grep で確認

### T1: `LearnChapter` 構造体 + `LEARN_CHAPTERS` 追加

v727000_tests の直後（ファイル末尾）に挿入:

```rust
// ── v72.8.0: fav learn ──────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LearnChapter {
    pub chapter: u32,
    pub title: &'static str,
    pub prompt: &'static str,
    pub hint: &'static str,
    pub expected_contains: &'static str,
}

pub static LEARN_CHAPTERS: &[LearnChapter] = &[
    LearnChapter {
        chapter: 1,
        title: "最初のパイプライン",
        prompt: "fn main(ctx: AppCtx) -> Result<Unit, String> を書いてみましょう",
        hint: "fn main(ctx: AppCtx) -> Result<Unit, String> { Ok(()) }",
        expected_contains: "fn main",
    },
    LearnChapter {
        chapter: 2,
        title: "型システムの力",
        prompt: "schema を使ってフィールドを定義してみましょう（例: schema Row { id: String }）",
        hint: "schema Row { id: String }",
        expected_contains: "schema",
    },
    LearnChapter {
        chapter: 3,
        title: "Rune を使ったデータ処理",
        prompt: "import rune でモジュールを読み込んでみましょう（例: import rune \"csv\"）",
        hint: "import rune \"csv\"",
        expected_contains: "import rune",
    },
    LearnChapter {
        chapter: 4,
        title: "AI パイプライン",
        prompt: "Llm.extract を呼び出す行を書いてみましょう",
        hint: "bind chunks <- Llm.extract(raw)",
        expected_contains: "Llm",
    },
    LearnChapter {
        chapter: 5,
        title: "分散実行",
        prompt: "par を使って並列ステージを書いてみましょう（例: par [LoadA, LoadB]）",
        hint: "par [LoadA, LoadB]",
        expected_contains: "par",
    },
];
```

`cargo build` でエラーがないことを確認。

### T2: `cmd_learn` 追加

`LEARN_CHAPTERS` の直後に追加:

```rust
pub fn cmd_learn() {
    println!("Favnir インタラクティブチュートリアル v1.0");
    println!();
    let stdin = std::io::stdin();
    for chapter in LEARN_CHAPTERS {
        println!("Chapter {}: {}", chapter.chapter, chapter.title);
        loop {
            println!("[{}/{}] {}", chapter.chapter, LEARN_CHAPTERS.len(), chapter.prompt);
            print!(">>> ");
            let _ = std::io::Write::flush(&mut std::io::stdout());
            let mut input = String::new();
            if stdin.lock().read_line(&mut input).is_err() || input.trim().is_empty() {
                println!("ヒント: {}", chapter.hint);
                continue;
            }
            if input.contains(chapter.expected_contains) {
                println!("✓ 正解！ 次へ進みます。");
                println!();
                break;
            } else {
                println!("ヒント: {}", chapter.hint);
            }
        }
    }
    println!("全章完了！ fav.dev/docs で次のステップへ。");
}
```

`cargo build` でエラーがないことを確認。

### T3: `main.rs` — `fav learn` コマンド追加

import リストに `cmd_learn` を追加し、`Some("learn")` アームを追加:

```rust
Some("learn") => {
    crate::driver::cmd_learn();
}
```

`cargo build` でエラーがないことを確認。

### T4: `v728000_tests` モジュール追加

`cmd_learn` の直後（ファイル末尾）に追加:

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

`cargo test v728000` で 2 件 pass することを確認。

### T5: バージョン更新

- T0 で確認した `"72.7.0"` 文字列の件数を記録しておく
- `fav/Cargo.toml`: `version = "72.7.0"` → `version = "72.8.0"`
- `driver.rs` 内の `version = \"72.7.0\"` を `version = \"72.8.0\"` に replace_all
- `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.7.0"` を `"72.8.0"` に replace_all
- `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.7.0"` を `"72.8.0"` に replace_all
- grep で残存 `72.7.0` をチェック — コメント・セクションヘッダー（`// ── v72.7.0:` 等）のみが残っていることを確認（これらは置換しない）

### T6〜T10: テスト・ドキュメント更新

- `cargo test v728000` — 2 件 pass
- `cargo test` — 3640 tests pass
- `CHANGELOG.md` に `## [v72.8.0]` エントリ追加
- `versions/current.md` 更新（進行中: v72.8.0 / 次: v72.9.0）
