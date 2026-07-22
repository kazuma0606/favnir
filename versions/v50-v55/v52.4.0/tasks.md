# Tasks: v52.4.0 — `fav explain --lineage` インタラクティブ HTML レポート

Status: COMPLETE
Date: 2026-07-21

---

## T0 — 事前確認

- [x] `cargo test` 3141 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `lineage.rs` に `render_lineage_html` が**存在しない**ことを確認（新規追加対象）
- [x] `main.rs` の `--lineage` ブロックに `-o` が**存在しない**ことを確認（新規追加対象）
- [x] `v52300_tests` に `cargo_toml_version_is_52_3_0` が**存在しない**ことを確認（削除対象なし）
- [x] `include_str!` パス確認（`fav/src/driver.rs` 起点）:
  - [x] `include_str!("lineage.rs")` → `fav/src/lineage.rs` ✓

## T1 — `render_lineage_html` 追加（lineage.rs）

- [x] `render_lineage_svg` の直後に `render_lineage_html` を挿入（`render_lineage_text` の前）
- [x] SVG 部分の構築:
  - [x] `svg_width = (n * 200 + 40).max(200)`、高さ 160px
  - [x] `<defs>` arrowhead marker（`id="arr"`）を追加
  - [x] 各 stage ノードを `<g class="node" onclick="showDetail(&quot;NAME&quot;)">` でラップ
    - [x] `entry.name` の `&` `<` `>` `"` を HTML エスケープ（onclick 属性の `&quot;`）
    - [x] `<rect x="{}" y="70" width="160" height="40" rx="4"/>` （fill/stroke は CSS が担当）
    - [x] stage 名 text（`y="86"`）、kind text（`y="100"`）
  - [x] pipeline エッジを `<line y1="90" y2="90">` + `marker-end="url(#arr)"` で描画
    - [x] `name_to_idx: HashMap<&str, usize>` を構築（`render_lineage_svg` と同パターン）
- [x] JS stages データ JSON 文字列の構築:
  - [x] effects: 空なら `"Pure"`、あれば `!X+!Y` 形式
  - [x] schema: `entry.schema.as_deref().unwrap_or("")`
  - [x] sources/sinks: `join(", ")`
  - [x] ダブルクォートを `.replace('"', "\\\"")` でエスケープ
- [x] `format!` で HTML 全体を組み立て:
  - [x] `<!DOCTYPE html>` を先頭に出力
  - [x] CSS で `.node rect:hover` のスタイルを定義
  - [x] `<div id="detail">` に初期メッセージ（"Click a stage node to see details."）
  - [x] `showDetail(name)` 関数でテーブル行を構築し `document.getElementById('detail').innerHTML` を更新
  - [x] schema / sources / sinks は空文字のとき行を省略
- [x] `format!` 内の `{{` `}}` エスケープを全箇所確認（CSS のブレース、JS のブレース）
- [x] `cargo build` → コンパイルエラーなし確認

## T2+T3 — `driver.rs` + `main.rs` 更新（同時実施必須）

> **注意**: T2 でシグネチャを変更すると T3 が完了するまで `cargo build` が通らない。
> T2 と T3 は一気通貫で実施し、両方完了後に `cargo build` を確認すること。

- [x] `pub use crate::lineage::` に `render_lineage_html` を追加
- [x] `cmd_explain_lineage` に `output: Option<&str>` 引数を追加:
  ```rust
  pub fn cmd_explain_lineage(
      file: Option<&str>, format: &str, show_dead: bool,
      with_schema: bool, output: Option<&str>,
  )
  ```
- [x] `match format` を `content` 変数パターンに変更:
  - [x] 各アームが `String` を返すように変更（`print!` を削除して値を返す）
  - [x] `"html"` アームを追加: `render_lineage_html(&report)`
  - [x] エラーメッセージの valid フォーマット一覧に `html` を追加
- [x] `output` による出力先振り分けを追加:
  - [x] `Some(out_path)` → `std::fs::write(out_path, &content)` + エラーハンドリング
  - [x] `None` → `print!("{}", content)`
- [x] `std::fs::write` は完全パス `std::fs::write(out_path, &content)` を使用（`use std::fs` は追加しない）
- [x] 既知の呼び出し箇所: `main.rs` の 1 箇所のみ（T3 で対応）
- [x] `rg "cmd_explain_lineage" fav/src/` で他に呼び出し箇所がないか確認
- [x] `cargo build` → コンパイルエラーなし確認

## T3 — `main.rs` 更新（T2 に続けて実施）

- [x] `--lineage` ブロックの変数宣言に `let mut output_file: Option<String> = None;` を追加
- [x] `while` ループの `match` に `-o` アームを追加（`other =>` catch-all の前に配置すること）:
  ```rust
  "-o" => {
      output_file = Some(args.get(i + 1).unwrap_or_else(|| {
          eprintln!("error: -o requires a file path");
          process::exit(1);
      }).clone());
      i += 2;
  }
  ```
  （`"--with-schema"` アームの直後に追加）
- [x] `cmd_explain_lineage` 呼び出しを更新:
  ```rust
  cmd_explain_lineage(file, &format, show_dead, with_schema, output_file.as_deref());
  ```
- [x] `cargo build` → コンパイルエラーなし確認

## T4 — `driver.rs` にテスト追加 + バージョン更新

- [x] `rg -n "v52300_tests" fav/src/driver.rs` で挿入位置を確認
- [x] `v52400_tests` モジュールを `v52300_tests` の直前に追加（3 件）:
  - [x] `lineage_html_output`（`include_str!` でソース確認）
  - [x] `lineage_html_has_stage_detail`（`id=\"detail\"` のみチェック、OR 条件不要）
  - [x] `lineage_html_renders_stage_node`（`render_lineage_html` を呼び出し実際の HTML 内容を確認）
- [x] `v52300_tests` に version テストなし → 削除対象なし（確認済み）
- [x] `fav/Cargo.toml` version → `"52.4.0"`
- [x] `cargo test` 実行 → 3144 passed, 0 failed を確認（テスト 3 件追加）
- [x] `cargo clippy -- -D warnings` クリーンを確認

## T5 — 後処理

- [x] `CHANGELOG.md` に v52.4.0 エントリ追加
- [x] `versions/current.md` を v52.4.0（3144 tests）に更新
- [x] `roadmap-v52.1-v53.0.md` の v52.4.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T5 全 `[x]`）
