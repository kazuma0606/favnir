# Favnir v10.0.0 実装計画 — OSS 公開準備完了

作成日: 2026-06-03

---

## 全体構成

変更が必要なファイルと依存関係：

```
Phase Z: 積み残し確認
  fav/self/compiler.fav  ← W004 実装確認（存在しなければ追加）
  versions/v10.0.0/known-limitations.md  ← par stack overflow 文書化

Phase A: fav new スキャフォールディング
  fav/src/backend/vm.rs  ← IO.make_dir_raw 追加（Rust 変更）
  fav/self/cli.fav       ← cmd_new 追加 + main ディスパッチ

Phase B: GitHub Actions CI
  .github/workflows/ci.yml  ← 新規作成

Phase C: ドキュメント
  CONTRIBUTING.md  ← リポジトリルートに新規作成
  CHANGELOG.md     ← リポジトリルートに新規作成
  LICENSE          ← 存在確認・なければ作成

Phase D: テスト + バージョン更新
  fav/src/driver.rs  ← v10_tests モジュール追加
  fav/Cargo.toml     ← version = "10.0.0"
  fav/self/cli.fav   ← run_version = "10.0.0"
```

---

## Phase Z — 積み残し確認

### Z-1: W004 実装確認

`compiler.fav` の lint セクションを検索：

```bash
grep -n "w004\|W004\|TooManyArgs\|lint_fn_w004" fav/self/compiler.fav
```

- **見つかった場合**: 実装済み。`memory/MEMORY.md` の「未実装タスク」欄から削除。
- **見つからない場合**: 以下を実装する。

```favnir
fn count_tuple_args(ty: Ty) -> Int =
  match ty {
    TTuple(elems) => List.length(elems)
    _             => 1
  }

fn lint_fn_w004(sd: StageDef) -> List<LintWarning> =
  if count_tuple_args(sd.in_ty) >= 4 {
    [LintWarning {
      code:    "W004"
      name:    sd.name
      message: "W004: stage " + sd.name + " の引数型が "
                + Int.to_string(count_tuple_args(sd.in_ty))
                + " 個です。レコード型へのまとめを検討してください"
    }]
  } else {
    []
  }
```

`lint_program` 内の stage チェックループに `lint_fn_w004(sd)` を追加。

### Z-2: known-limitations.md 作成

```markdown
# Favnir 既知の制限事項

## compiler.fav pipeline での par コンパイル

### 症状
`compile_file_to_bytes`（compiler.fav pipeline）で
par を含む seq をコンパイルすると Rust stack overflow が発生する。

### 根本原因
`List.fold` + lambda を含む stage body の再帰コンパイルが
Favnir VM のコールスタックを深くしすぎる（Rust デフォルト stack size: 8MB）。

### 回避策
par を含む seq は Rust pipeline（`fav run --legacy`）でコンパイルする。
または `IO.par_execute_raw` を直接呼び出すコードに書き換える。

### 将来の修正方針
- Rust のスレッドスタックサイズを `RUST_MIN_STACK` 環境変数で拡大
- またはコンパイラ内の再帰を trampoline パターンに書き換える
- v10.1.0 以降で対応予定
```

---

## Phase A — `fav new` スキャフォールディング

### A-1: IO.make_dir_raw の追加（vm.rs）

`vm.rs` の IO namespace builtin に追加。

```rust
// IO.make_dir_raw(path: String) -> Unit !Io
"make_dir_raw" => {
    let path = expect_string(&args[0])?;
    std::fs::create_dir_all(&path)
        .map_err(|e| format!("IO.make_dir_raw: {}", e))?;
    Ok(VMValue::Unit)
}
```

### A-2: cmd_new の実装（cli.fav）

`cli.fav` の末尾に追加：

```favnir
fn new_fav_toml(name: String) -> String =
  "[project]\nname = \"" + name + "\"\nversion = \"0.1.0\"\nsrc = \"src\"\n"

fn new_main_fav() -> String =
  "type Order = { id: Int  item: String  amount: Float }\n\n"
  + "stage ParseOrder: String -> Order = |s| {\n"
  + "  Order { id: 1  item: s  amount: 0.0 }\n"
  + "}\n\n"
  + "stage FormatOrder: Order -> String = |o| {\n"
  + "  \"Order#\" + Int.to_string(o.id) + \": \" + o.item\n"
  + "}\n\n"
  + "seq ProcessOrder = ParseOrder |> FormatOrder\n"

fn new_gitignore() -> String =
  "*.fvc\n.fav_cache/\n"

fn cmd_new(name: String) -> Unit !Io = {
  let root    = name
  let src_dir = name + "/src"
  IO.make_dir_raw(src_dir)
  IO.write_file_raw(root + "/fav.toml",    new_fav_toml(name))
  IO.write_file_raw(src_dir + "/main.fav", new_main_fav())
  IO.write_file_raw(root + "/.gitignore",  new_gitignore())
  IO.print("Created project: " + name)
}
```

### A-3: main ディスパッチへの追加（cli.fav）

`run_command` の match に `"new"` 分岐を追加：

```favnir
"new" => {
  match List.first(args) {
    None       => IO.print("Usage: fav new <name>")
    Some(name) => cmd_new(name)
  }
}
```

---

## Phase B — GitHub Actions CI

### B-1: .github/workflows/ci.yml の作成

`spec.md` の YAML をそのまま配置。

### B-2: YAML 構文確認

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo OK
```

---

## Phase C — ドキュメント

### C-1: LICENSE 確認

```bash
ls /c/Users/yoshi/favnir/LICENSE 2>/dev/null && echo exists || echo missing
```

存在しない場合は MIT テキストを配置する。

### C-2: CONTRIBUTING.md

リポジトリルート（`/c/Users/yoshi/favnir/`）に配置。
spec.md の「3. CONTRIBUTING.md」節の内容で作成。

### C-3: CHANGELOG.md

リポジトリルート（`/c/Users/yoshi/favnir/`）に配置。
v10.0.0〜v4.0.0（主要マイルストーンのみ）を降順でまとめる。

---

## Phase D — テスト + バージョン更新

### D-1: v10_tests の追加（driver.rs）

```rust
#[cfg(test)]
mod v10_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn fav_new_creates_project_structure() {
        let dir = TempDir::new().unwrap();
        let name = "testproject";
        // cmd_new を経由してプロジェクト生成
        // fav.toml / src/main.fav / .gitignore の存在を確認
        let root = dir.path().join(name);
        // ... テスト実装
        assert!(root.join("fav.toml").exists());
        assert!(root.join("src/main.fav").exists());
        assert!(root.join(".gitignore").exists());
    }

    #[test]
    fn fav_new_generated_project_runs() {
        let dir = TempDir::new().unwrap();
        // cmd_new → fav run src/main.fav が Ok を返すことを確認
        // ...
    }
}
```

実際のテストは CLI 経由ではなく `IO.make_dir_raw` + `IO.write_file_raw` の
Favnir pipeline 実行を通じて確認する。

### D-2: バージョン更新

- `fav/Cargo.toml`: `version = "9.13.0"` → `"10.0.0"`
- `fav/self/cli.fav`: `"9.13.0"` → `"10.0.0"`

---

## 実装順序と依存関係

```
Z-1（W004確認） → MEMORY.md 更新
Z-2（known-limitations）

A-1（vm.rs make_dir_raw）
  ↓
A-2（cli.fav cmd_new）
  ↓
A-3（cli.fav dispatch）
  ↓
D-1（tests）

B-1（ci.yml）← 独立、並行可
C-1（LICENSE）← 独立、並行可
C-2（CONTRIBUTING.md）← 独立、並行可
C-3（CHANGELOG.md）← 独立、並行可

D-2（バージョン更新）← 最後
```

---

## 注意点

### IO.make_dir_raw が既存か確認

vm.rs に `make_dir_raw` が既に存在する可能性がある（過去実装のチェック）：

```bash
grep -n "make_dir" fav/src/backend/vm.rs
```

存在すれば A-1 をスキップ。

### cli.fav の文字列連結

Favnir の文字列リテラルでは `\n` がエスケープシーケンスとして使える。
ファイルテンプレートは複数の `+` 連結か `String.join` で構築する。

### driver.rs のテスト設計

`cmd_new` は CLI 経由なので、`fav run` の Favnir pipeline で `cmd_new("name")` を
呼び出すスタイルのテストを書くのが自然。または
Rust 側で `IO.make_dir_raw` / `IO.write_file_raw` を直接呼び出すヘルパーを使う。

簡単な選択肢: `cmd_new` の代わりに Favnir source を tempdir で実行してファイル生成を確認。

### fav fmt --check の CI 通過

`fav fmt --check self/compiler.fav` が CI で差分なしを返すように、
実装前に `fav fmt self/compiler.fav` を適用しておく（必要な場合）。
