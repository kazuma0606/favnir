# Plan: v52.4.0 — `fav explain --lineage` インタラクティブ HTML レポート

Status: PLANNED
Date: 2026-07-21

---

## 実装順序

### Step 1 — `render_lineage_html` 追加（lineage.rs）

- ファイル: `fav/src/lineage.rs`
- 挿入位置: `render_lineage_svg` の直後（`render_lineage_text` の前）

**実装手順**:
1. 関数シグネチャ: `pub fn render_lineage_html(report: &LineageReport) -> String`
2. `render_lineage_svg` と同じ座標計算（幅 `(n * 200 + 40).max(200)`、高さ 160px）を使用
3. SVG 部分の構築:
   - `<defs>` arrowhead marker（`render_lineage_svg` と同じ `#arr`）
   - 各 stage ノードを `<g class="node" onclick="showDetail(&quot;NAME&quot;)">` でラップ
     - `<rect x="{}" y="70" width="160" height="40" rx="4"/>`
     - stage 名 text（y=86）、kind text（y=100）
   - pipeline エッジを `<line y1="90" y2="90">` で描画（`render_lineage_svg` と同じ `name_to_idx` マップを使用）
4. JS `stages` データ JSON 文字列の構築:
   - 各エントリの effects: 空なら `"Pure"`、あれば `!X+!Y` 形式
   - `entry.schema.as_deref().unwrap_or("")`
   - sources/sinks は `join(", ")`
   - ダブルクォートは `.replace('"', "\\\"")` でエスケープ
5. `format!` で HTML 全体を組み立て（`r#"..."#` raw string リテラルを使用）
   - CSS は minified inline（`body{{...}}` — `format!` 内では `{` を `{{` でエスケープ）
   - `showDetail` 関数は JS でインライン実装

**Clippy 注意点**:
- `format!` 内での `{{` `}}` エスケープを徹底する
- `.replace('"', "\\\"")` は `str::replace` であり Clippy 警告なし
- `name_to_idx` は `HashMap<&str, usize>` — ライフタイムに注意（既存 `render_lineage_svg` と同パターン）

**`id="detail"` の書き方**:
`format!` マクロ内のダブルクォートは `\"` でエスケープ。raw string `r#"..."#` を使う場合は
`id="detail"` がそのまま書けるが、raw string 内の `{}` プレースホルダが機能することを確認する。

### Step 2 — `driver.rs` 更新

- ファイル: `fav/src/driver.rs`

**2a. `pub use crate::lineage::` に追加**:
```rust
render_lineage_html,
```
（`render_lineage_dot_with_schema` の直後に追加）

**2b. `cmd_explain_lineage` シグネチャ更新**:
```rust
pub fn cmd_explain_lineage(
    file: Option<&str>,
    format: &str,
    show_dead: bool,
    with_schema: bool,
    output: Option<&str>,
)
```

**2c. `match format` を `content` 変数パターンに変更**:
現在の各アームが `print!("{}", ...)` を直接呼んでいるものを、
まず `content` 変数に結果を格納し、その後 `output` で出力先を振り分ける:

```rust
let content = match format {
    "json"    => render_lineage_json(&report),
    "mermaid" => render_lineage_mermaid_with_schema(&report, show_dead, with_schema),
    "d2"      => render_lineage_d2(&report),
    "dot"     => render_lineage_dot_with_schema(&report, with_schema),
    "svg"     => render_lineage_svg(&report),
    "html"    => render_lineage_html(&report),
    "text"    => render_lineage_text(&report, path),
    other     => {
        eprintln!("error: unknown format '{}'. valid: text, json, mermaid, d2, dot, svg, html", other);
        process::exit(1);
    }
};
if let Some(out_path) = output {
    std::fs::write(out_path, &content).unwrap_or_else(|e| {
        eprintln!("error: could not write to '{}': {}", out_path, e);
        process::exit(1);
    });
} else {
    print!("{}", content);
}
```

**2d. シグネチャ変更の影響確認**:
- `rg "cmd_explain_lineage" fav/src/` で呼び出し箇所を確認
- 既知: `main.rs` line 852 の 1 箇所 → Step 3 で対応

### Step 3 — `main.rs` 更新

- ファイル: `fav/src/main.rs`

**3a. 変数宣言に追加**（`with_schema` の直後）:
```rust
let mut output_file: Option<String> = None;
```

**3b. `while` ループの `match` に追加**（`"--with-schema"` アームの直後）:
```rust
"-o" => {
    output_file = Some(
        args.get(i + 1)
            .unwrap_or_else(|| {
                eprintln!("error: -o requires a file path");
                process::exit(1);
            })
            .clone()
    );
    i += 2;
}
```

**3c. `cmd_explain_lineage` 呼び出し更新**:
```rust
cmd_explain_lineage(file, &format, show_dead, with_schema, output_file.as_deref());
```

**注意**: `-o` フラグの処理で `i += 2` を忘れると次のトークンが input file として解釈される。

### Step 4 — `driver.rs` にテスト追加 + バージョン更新

- `v52400_tests` モジュールを `v52300_tests` の直前に追加（2 件）
- `fav/Cargo.toml` version → `"52.4.0"`
- `cargo test` → 3143 passed, 0 failed を確認
- `cargo clippy -- -D warnings` クリーンを確認

### Step 5 — 後処理

- `CHANGELOG.md` に v52.4.0 エントリ追加
- `versions/current.md` を v52.4.0（3143 tests）に更新
- `versions/roadmap/roadmap-v52.1-v53.0.md` の v52.4.0 実績欄を更新
- `tasks.md` を COMPLETE に更新（T0〜T5 全 `[x]`）

---

## 注意事項

- `render_lineage_html` の `format!` マクロは CSS / JS のブレースが多いため `{{` `}}` エスケープが必要。
  raw string `r#"..."#` を使う場合でも `format!` の引数ではプレースホルダを正しく展開する。
- `onclick="showDetail(&quot;NAME&quot;)"` — HTML 属性内のダブルクォートは `&quot;` でエスケープする。
  `entry.name` に `<>&"` が含まれる可能性があるため `html_escape_name` ヘルパーを定義しても良い
  （ただし通常の stage 名は英数字+アンダースコアのみなので、必須ではない）。
- `std::fs::write` は `use std::fs` が必要 — driver.rs に既存の `std::fs` import があるか確認すること。
  `use std::fs` がない場合は完全パス `std::fs::write(...)` を使う（import 追加は避ける方針）。
- `"text"` フォーマットは複数パスに対して複数回 `render_lineage_text` を呼ぶ設計。
  `output` が `Some` の場合に複数パスで複数回 `fs::write` を呼ぶと上書きになる。
  今バージョンでは `--format html -o <file>` は単一ファイル指定を前提とし、
  複数ファイルの場合は最後のファイルで上書きされることを仕様として受け入れる。
  （将来 v52.x で改善する余地あり）
