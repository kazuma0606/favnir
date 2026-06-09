# Favnir v13.1.0 実装計画

Date: 2026-06-09

---

## Phase A — AST + parser.rs: interface 継承構文

### A-1: `InterfaceDef` に `parent: Option<String>` を追加

対象ファイル: `fav/src/ast.rs`

```rust
pub struct InterfaceDef {
    pub name:   String,
    pub parent: Option<String>,   // 追加
    pub fields: Vec<InterfaceField>,
    pub span:   Span,
}
```

`InterfaceDef` を生成する全箇所に `parent: None` を追加（既存コードへの影響）。

### A-2: `parse_interface_def` に `: ParentName` 解析を追加

対象ファイル: `fav/src/frontend/parser.rs`

現在の構文: `interface Name { fields... }`
変更後の構文: `interface Name [: ParentName] { fields... }`

```rust
// interface キーワード消費後
let name = self.expect_ident()?;
let parent = if self.peek_token() == Token::Colon {
    self.advance(); // `:` 消費
    Some(self.expect_ident()?)
} else {
    None
};
self.expect_token(Token::LBrace)?;
// フィールドパース...
InterfaceDef { name, parent, fields, span }
```

### A-3: fmt.rs の interface フォーマット更新

対象ファイル: `fav/src/fmt.rs`

`format_interface_def` で `parent` が `Some(p)` の場合に `: p` を出力：

```rust
format!("interface {}{} {{",
    def.name,
    def.parent.as_ref().map(|p| format!(": {}", p)).unwrap_or_default()
)
```

---

## Phase B — checker.rs: 継承フィールド解決 + E0019

### B-1: interface 継承フィールドの解決ロジック

対象ファイル: `fav/src/middle/checker.rs`

`resolve_interface_fields` 関数を追加（または既存の lookup に統合）：

```rust
fn resolve_interface_fields<'a>(
    name: &str,
    interfaces: &'a HashMap<String, InterfaceDef>,
    depth: u8,
) -> Vec<&'a InterfaceField> {
    if depth > 16 { return vec![]; } // 循環ガード
    let Some(def) = interfaces.get(name) else { return vec![]; };
    let mut fields: Vec<&InterfaceField> = def.fields.iter().collect();
    if let Some(parent) = &def.parent {
        fields.extend(resolve_interface_fields(parent, interfaces, depth + 1));
    }
    fields
}
```

フィールドアクセス `expr.field` の型推論時、interface 型として解決する箇所に適用。

### B-2: E0019 循環継承検出

対象ファイル: `fav/src/middle/checker.rs`

checker が全 interface 定義を収集した後に循環チェックを実行：

```rust
fn check_interface_cycles(
    interfaces: &HashMap<String, InterfaceDef>,
) -> Vec<Diagnostic> {
    let mut errors = vec![];
    for name in interfaces.keys() {
        let mut visited = HashSet::new();
        let mut cur = name.as_str();
        while let Some(def) = interfaces.get(cur) {
            if !visited.insert(cur) {
                errors.push(Diagnostic {
                    code: "E0019",
                    message: format!("circular interface inheritance detected: {}", cur),
                    // span: 継承宣言のスパン
                });
                break;
            }
            match &def.parent {
                Some(p) => cur = p.as_str(),
                None => break,
            }
        }
    }
    errors
}
```

`fav check` の型チェックフロー（`check_program`）の先頭に追加。

---

## Phase C — compiler.fav / checker.fav: セルフホスト対応

### C-1: compiler.fav の `parse_interface_def` 更新

対象ファイル: `fav/self/compiler.fav`

既存の `parse_interface_def` 関数を修正：
- interface 名をパース後、次トークンが `:` なら ParentName を読む
- `InterfaceDef { name, parent, fields }` で返す（parent: Option<String>）

`InterfaceDef` レコード型に `parent` フィールドを追加（`String` として保存、`""` = なし）：

```favnir
type InterfaceDef = {
  name:   String
  parent: String   // "" = 継承なし
  fields: List<InterfaceField>
}
```

### C-2: checker.fav の `resolve_interface_field` 更新

対象ファイル: `fav/self/checker.fav`

`infer_field_access` または `lookup_interface_field` に継承チェーン解決を追加：

```favnir
fn lookup_interface_field_recursive(
  ifaces: Map<String, InterfaceDef>,
  name: String,
  field: String,
  depth: Int
) -> Option<String> {
  match Map.get(ifaces, name) {
    None => Option.none()
    Some(def) =>
      match find_field(def.fields, field) {
        Some(ty) => Option.some(ty)
        None =>
          if String.length(def.parent) == 0 { Option.none() }
          else if depth > 16 { Option.none() }
          else { lookup_interface_field_recursive(ifaces, def.parent, field, depth + 1) }
      }
  }
}
```

### C-3: `fav check` / `fav fmt --check` でセルフホスト検証

```bash
./target/debug/fav check self/compiler.fav
./target/debug/fav check self/checker.fav
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

## Phase D — lint.rs: W008 ambient effect 警告

### D-1: ambient namespace 定数を定義

対象ファイル: `fav/src/lint.rs`（または新規 `fav/src/ambient.rs`）

```rust
const AMBIENT_NAMESPACES: &[&str] = &[
    "IO", "Postgres", "AWS", "Snowflake",
    "Http", "Grpc", "Llm", "Queue", "Cache",
    "Slack", "Email",
];

// 副作用のある Gen 関数（乱数生成）
const AMBIENT_GEN_FNS: &[&str] = &["uuid_raw", "uuid_v7_raw", "nano_id"];
```

### D-2: `check_ambient_effects` 関数を実装

対象ファイル: `fav/src/lint.rs`

```rust
pub fn check_ambient_effects(program: &Program) -> Vec<LintWarning> {
    let mut warnings = vec![];
    // AST を walk して Call { ns: "IO", fn: "println", ... } のパターンを検出
    // ctx.io.println(...) 形式（フィールドアクセス経由）は除外
    for stmt in &program.stmts {
        collect_ambient_calls(stmt, &mut warnings);
    }
    warnings
}
```

`LintWarning` に `code: "W008"` を追加（既存の `W001`〜`W007` と同形式）。

### D-3: W008 のヘルプテキストを `get_help_text` に追加

対象ファイル: `fav/src/driver.rs`

```rust
"W008" => &[
    "pass the capability as a ctx argument: `ctx.io.println(...)`",
    "ambient effects will become E0023 (error) in v14.0",
],
```

---

## Phase E — driver.rs: `--ambient` フラグ + `--report`

### E-1: `cmd_check` に `--ambient` フラグを追加

対象ファイル: `fav/src/driver.rs` + `fav/src/main.rs`

`cmd_check(file, no_warn, legacy_check, json, show_types, strict, ambient: bool)` に引数追加。

`ambient == true` のとき `check_ambient_effects(&program)` を呼び出し、
W008 を通常の警告フォーマットで出力する。

### E-2: `--report` フラグで Markdown レポート生成

`--ambient --report` 時は W008 を Markdown 形式で `lab/audit/w008-ambient.md` に書き出す：

```rust
fn write_ambient_report(warnings: &[LintWarning], target_file: &str) -> std::io::Result<()> {
    // lab/audit/ ディレクトリを作成（なければ）
    // Markdown テーブル形式で W008 を出力
}
```

### E-3: `main.rs` の check ディスパッチ更新

```rust
let mut ambient = false;
let mut report  = false;
// ...
"--ambient" => { ambient = true; i += 1; }
"--report"  => { report  = true; i += 1; }
// ...
cmd_check(file, no_warn, legacy_check, json, show_types, strict, ambient, report)
```

---

## Phase F — テスト追加

### F-1: Rust ユニットテスト（`fav/src/driver.rs` 末尾）

```rust
#[cfg(test)]
mod v131000_tests {
    #[test]
    fn version_is_13_1_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "13.1.0");
    }

    #[test]
    fn interface_inheritance_parsed() {
        use crate::frontend::parser::Parser;
        let src = "interface CommonCtx { io: Io }\ninterface LoadCtx: CommonCtx { db: DbRead }";
        let prog = Parser::parse_str(src, "test.fav").expect("parse error");
        let load_ctx = prog.interfaces.iter().find(|i| i.name == "LoadCtx").unwrap();
        assert_eq!(load_ctx.parent.as_deref(), Some("CommonCtx"));
    }

    #[test]
    fn e0019_circular_interface_detected() {
        // "interface A: B {}" + "interface B: A {}" → E0019
    }

    #[test]
    fn w008_ambient_io_println_detected() {
        // IO.println(...) + --ambient → W008
    }

    #[test]
    fn w008_no_flag_no_warning() {
        // IO.println(...) without --ambient → no W008
    }
}
```

### F-2: バージョン更新

対象ファイル: `fav/Cargo.toml`

```toml
version = "13.1.0"
```

---

## Phase G — ビルド・テスト・コミット

### G-1: cargo build

```bash
cd fav && cargo build
```

### G-2: cargo test

```bash
cargo test
```

### G-3: self-check

```bash
./target/debug/fav check self/compiler.fav
./target/debug/fav check self/checker.fav
./target/debug/fav lint --deny-warnings self/compiler.fav
./target/debug/fav lint --deny-warnings self/checker.fav
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

### G-4: W008 調査レポート生成（手動確認）

```bash
./target/debug/fav check --ambient --report self/compiler.fav
./target/debug/fav check --ambient --report self/checker.fav
cat ../lab/audit/w008-ambient.md
```

### G-5: git commit + push

```bash
git add -p
git commit -m "feat: v13.1.0 — interface 継承 + W008 ambient effect 警告"
git push
```

### G-6: CI 確認

```bash
gh run watch
```

---

## 実装上の注意

### 1. `InterfaceDef` の既存リテラル

AST に `parent` フィールドを追加すると、`checker.rs` / `resolver.rs` 等で
`InterfaceDef { name, fields, span }` と書かれている箇所がすべてコンパイルエラーになる。
`parent: None` を追加するだけだが、漏れなく対応する。

### 2. compiler.fav のレコード型変更

`self/compiler.fav` 内の `InterfaceDef` レコード型に `parent: String` を追加すると、
それを生成している全箇所に `parent: ""` を追加する必要がある。

### 3. W008 は `fav lint` には含めない

`fav lint` は W001〜W007 のみ対象。W008 は `fav check --ambient` 専用。
`lint_program` 関数は変更しない。

### 4. E0019 のスパン情報

`InterfaceDef` に `span` フィールドがすでにある場合、
親インターフェース宣言のスパンを別途保持する必要があるかもしれない。
最初の実装はメッセージにインターフェース名のみ表示し、スパンは後回しでよい。

### 5. `lab/audit/` ディレクトリ

`--report` フラグ実行前に `lab/audit/` が存在しない場合、
`std::fs::create_dir_all` で作成する。
生成ファイルは `.gitignore` に追加しない（バージョン管理で移行進捗を追跡する）。
