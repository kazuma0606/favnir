# v24.0.0 実装計画 — VM in Favnir マイルストーン宣言

## 前提確認

v24.0.0 は Rust 2 ファイル（driver.rs + main.rs）+ ドキュメント変更のみ。vm.fav 変更なし。

### 実装前チェック

```bash
grep -n "version = " fav/Cargo.toml
# → "23.8.0" であること

grep -n "mod v238000_tests\|mod v240000_tests" fav/src/driver.rs | head -5
# → v240000_tests が未存在であること

grep -n "run_with_vm\|--vm" fav/src/driver.rs fav/src/main.rs | head -5
# → 全 0 件であること（未実装）

grep -n "VM in Favnir" README.md | head -3
# → 0 件であること
```

---

## T0: 事前確認

```bash
# exec_artifact_main / build_artifact の可視性確認
grep -n "^fn build_artifact\|^fn exec_artifact_main\|^pub fn" fav/src/driver.rs | head -10

# main.rs の "run" アームで --vm フラグ挿入位置確認
grep -n "\"--debug\"\|\"--precompiled\"\|cmd_run_debug\|cmd_run_precompiled" fav/src/main.rs | head -10
```

---

## T1: `fav/src/driver.rs` — `run_with_vm` 追加

`cmd_run_precompiled` 関数（約 31633 行）の直後に追加する。

```rust
/// v24.0.0: vm.fav 経由でバイトコードを実行する。
/// vm_src: vm.fav のソースコード（`include_str!("../self/vm.fav")` 等）
/// bytecode_hex: 実行するバイトコードの hex 文字列（例: "12000016"）
/// globals_entries: globals マップに設定する (index, value_str) のリスト
///   例: &[(0, "hello")] → globals[0] = VMStr("hello")
pub fn run_with_vm(
    vm_src: &str,
    bytecode_hex: &str,
    globals_entries: &[(usize, &str)],
) -> Result<Value, String> {
    let mut globals_setup = String::new();
    for (idx, val) in globals_entries {
        let escaped = val.replace('\\', "\\\\").replace('"', "\\\"");
        globals_setup.push_str(&format!(
            "  bind _ <- Mut.set(globals, {}, VMStr(\"{}\"))\n",
            idx, escaped
        ));
    }
    let src = format!(
        r#"{vm_src}
public fn main() -> String {{
  bind globals <- Mut.map()
{globals_setup}  bind hex_r <- Bytes.from_hex("{bytecode_hex}")
  match hex_r {{
    ok(bytes) => {{
      bind run_r <- vm_run_named(bytes, globals)
      match run_r {{
        ok(v)  => vmval_display(v)
        err(e) => e
      }}
    }}
    err(e) => e
  }}
}}"#,
        vm_src = vm_src,
        globals_setup = globals_setup,
        bytecode_hex = bytecode_hex,
    );
    let tokens = crate::frontend::lexer::Lexer::new(&src, "vm_runner.fav")
        .tokenize()
        .map_err(|e| format!("lex: {:?}", e))?;
    let prog = crate::frontend::parser::Parser::new(tokens)
        .parse_program()
        .map_err(|e| format!("parse: {:?}", e))?;
    let artifact = build_artifact(&prog);
    exec_artifact_main(&artifact, None).map_err(|e| format!("exec: {:?}", e))
}
```

> **注意**: `build_artifact` と `exec_artifact_main` は driver.rs 内の private 関数。
> `run_with_vm` を同じファイルに追加することで直接呼び出し可能。

---

## T2: `fav/src/main.rs` — `--vm / --hex` フラグ追加

`fav run` の `--precompiled` チェックブロック（約 354 行）の直後に追加する。

```rust
            // ── v24.0.0: fav run --vm <path> --hex <hex> ─────────────────────
            if let Some(vm_pos) = args.iter().position(|a| a == "--vm") {
                let vm_path = args.get(vm_pos + 1).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: --vm requires a path argument");
                    process::exit(1);
                });
                let hex_pos = args.iter().position(|a| a == "--hex").unwrap_or_else(|| {
                    eprintln!("error: --vm requires --hex <bytecode_hex>");
                    process::exit(1);
                });
                let bytecode_hex = args.get(hex_pos + 1).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: --hex requires a hex string argument");
                    process::exit(1);
                });
                let vm_src = std::fs::read_to_string(vm_path).unwrap_or_else(|e| {
                    eprintln!("error: cannot read {}: {}", vm_path, e);
                    process::exit(1);
                });
                match driver::run_with_vm(&vm_src, bytecode_hex, &[]) {
                    Ok(v) => println!("{}", v),
                    Err(e) => {
                        eprintln!("error: {}", e);
                        process::exit(1);
                    }
                }
                return;
            }
            // ─────────────────────────────────────────────────────────────────
```

**挿入位置:** `--precompiled` ブロックの `return;` 直後のコメント行（`// ─────────────────────────────────────────────────────────────────`）の**直後**、
`let mut db_path: Option<String> = None;` 宣言（`// Parse --db / --legacy / ...` while ループ開始）の前。

---

## T1 + T2 事後確認

```bash
cargo check --bin fav
# → エラー 0 であること

# 後方互換性確認
cargo test v238000 --bin fav
# → 5/5 PASS（version_is_23_8_0 は削除済みのため 5 件）
```

---

## T3: `fav/src/driver.rs` — v240000_tests 追加

### T3-1: `v238000_tests::version_is_23_8_0` を削除（T4-1 より前に必須）

```rust
    #[test]
    fn version_is_23_8_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"23.8.0\""), "Cargo.toml should have version 23.8.0");
    }
```
この関数ごと削除する。

### T3-2: `v240000_tests` モジュールを `v238000_tests` の直後に追加

```rust
// ── v240000_tests (v24.0.0) — VM in Favnir マイルストーン宣言 ──────────────
#[cfg(test)]
mod v240000_tests {
    use super::*;

    #[test]
    fn version_is_24_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"24.0.0\""), "Cargo.toml should have version 24.0.0");
    }

    #[test]
    fn run_with_vm_hello() {
        // run_with_vm: LoadGlobal(0)="hello" + Return → vmval_display = "hello"
        // hex: "12000016"
        //   12 00 00  LoadGlobal(0)  → push VMStr("hello")
        //   16        Return          → VMStr("hello")
        let vm_src = include_str!("../self/vm.fav");
        let result = run_with_vm(vm_src, "12000016", &[(0, "hello")])
            .expect("run_with_vm should succeed");
        assert_eq!(result, crate::value::Value::Str("hello".to_string()),
            "run_with_vm hello test should return \"hello\"");
    }

    #[test]
    fn run_with_vm_string_trim() {
        // run_with_vm: LoadGlobal(0)="String" + GetField(1)="trim" + LoadGlobal(2)=" hi " + Call(1) + Return
        // hex: "12000040010012020015010016"
        //   12 00 00  LoadGlobal(0)  → push VMStr("String")
        //   40 01 00  GetField(1)    → push VMStr("String.trim")
        //   12 02 00  LoadGlobal(2)  → push VMStr(" hi ")
        //   15 01 00  Call(1)        → call_builtin("String.trim") → VMStr("hi")
        //   16        Return          → VMStr("hi")
        let vm_src = include_str!("../self/vm.fav");
        let result = run_with_vm(
            vm_src,
            "12000040010012020015010016",
            &[(0, "String"), (1, "trim"), (2, " hi ")],
        )
        .expect("run_with_vm string trim should succeed");
        assert_eq!(result, crate::value::Value::Str("hi".to_string()),
            "run_with_vm String.trim(\" hi \") should return \"hi\"");
    }

    #[test]
    fn changelog_has_v24_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v24.0.0]"), "CHANGELOG.md should have [v24.0.0] entry");
    }

    #[test]
    fn readme_has_vm_in_favnir() {
        let readme = include_str!("../../README.md");
        assert!(readme.contains("VM in Favnir"), "README.md should have VM in Favnir section");
    }
}
```

---

## T4: Cargo.toml + CHANGELOG + README + benchmarks

> **注意**: T3-1 の `version_is_23_8_0` 削除完了後に Cargo.toml を更新すること。

### T4-1: `fav/Cargo.toml` バージョン更新

```
version = "23.8.0" → "24.0.0"
```

### T4-2: `CHANGELOG.md` 先頭に v24.0.0 エントリ追加

```markdown
## [v24.0.0] — 2026-06-23 — VM in Favnir マイルストーン宣言

### Added
- `driver::run_with_vm(vm_src, bytecode_hex, globals_entries)` — vm.fav 経由でバイトコードを実行する公開 API
- `fav run --vm <path> --hex <hex>` CLI フラグ — 端末から vm.fav 経由でバイトコードを直接実行

### Notes
- VM in Favnir マイルストーン宣言（v23.1〜v24.0 の達成を宣言）
  - v23.1: Bytes 型 / v23.2: ビット演算 / v23.3: Mut<T>
  - v23.4〜v23.8: vm.fav Phase 1〜5（デコード・実行ループ・制御フロー・builtin・GetField）
- ロードマップ完了条件 1〜3・5 を達成；条件 4（500件超テスト）は Phase 6 以降
```

### T4-3: `README.md` に "VM in Favnir" セクション追加

`v23.0.0` 現在の状態セクションの直上に以下を挿入:

```markdown
**v24.0.0（2026-06-23）— VM in Favnir マイルストーン宣言**

テスト: **1931 件以上**

### VM in Favnir 達成実績（v23.x）

| 機能 | バージョン | 概要 |
|---|---|---|
| Bytes 型 | v23.1.0 | `Bytes.from_hex / get / slice / concat / to_utf8 / read_u16_le` 等 13 操作 |
| ビット演算 | v23.2.0 | `Int.bit_and / bit_or / bit_xor / bit_not / shift_left / shift_right` + 16進数リテラル |
| Mut<T> 可変コレクション | v23.3.0 | `Mut.list / map / push / pop / get / set / peek` — VM スタック・ローカル変数テーブル |
| vm.fav Phase 1 | v23.4.0 | バイトコードデコード（27 opcode）、`Bytes.read_u16_le` |
| vm.fav Phase 2 | v23.5.0 | スタックベース実行ループ・VMVal 型（3 バリアント） |
| vm.fav Phase 3 | v23.6.0 | 制御フロー（Jump/JumpIfFalse）・ローカル変数（LoadLocal/StoreLocal）・残余演算 |
| vm.fav Phase 4 | v23.7.0 | builtin ディスパッチ（call_builtin）・VMStr 追加・LoadGlobal/Call |
| vm.fav Phase 5 | v23.8.0 | GetField・collect_args・vmval_display・任意 argc 対応 |
| `fav run --vm` CLI | v24.0.0 | `fav run --vm <vm_path> --hex <hex>` で vm.fav 経由実行 |

---

```

### T4-4: `benchmarks/v24.0.0.json` 作成

```json
{
  "version": "24.0.0",
  "date": "2026-06-23",
  "test_count": 0,
  "feature": "VM in Favnir マイルストーン宣言",
  "metrics": {
    "vm_fav_phases": 5,
    "milestone": "VM in Favnir",
    "roadmap_conditions_met": 4,
    "new_pub_fns": 1
  }
}
```

> `test_count` は最終 `cargo test --bin fav` 後に更新。

---

## 実装順序

```
T0（事前確認）
T1（driver.rs: run_with_vm 追加）
T2（main.rs: --vm / --hex フラグ追加）
cargo check → エラー 0 確認
T3-1（version_is_23_8_0 削除）← T4-1 より前に必須
T3-2（v240000_tests 追加）
cargo test v240000 → 5/5 PASS 確認
T4-1（version 更新）← T3-1 完了後
T4-2〜4（CHANGELOG / README / benchmarks）
cargo test --bin fav → リグレッションなし確認（1926 件以上）
```

---

## リスク対応表

| リスク | 検出方法 | 対応 |
|---|---|---|
| `run_with_vm` の format! で vm.fav 中の `{` `}` が問題になる | 問題なし | vm.fav ソースは format! の**引数値**として渡されるため、その中の `{` `}` は format! テンプレートとして再解釈されない。`execute_hello_via_vm` テストで動作実証済み |
| `run_with_vm` の format! でエスケープ漏れ | テスト失敗 | `val.replace('\\', "\\\\").replace('"', "\\\"")` で対処済み |
| `build_artifact` / `exec_artifact_main` が private で呼べない | cargo check エラー | 同一ファイル（driver.rs）内に追加するため問題なし |
| `--vm` フラグが `--debug` より後に評価される | 機能影響なし | `--debug` + `--vm` 同時指定は未定義（`--debug` が優先される）。ユーザーが同時指定する想定はない。必要なら `--vm` を `--debug` チェックの前に移動する |
| `--vm` フラグが他フラグより前に評価される | 不要 | `args.iter().position(|a| a == "--vm")` はリスト全体を検索 |
| `globals_entries` の順序が `Mut.set` 複数回の順序に影響する | テスト失敗 | Mut.map はキー=Int でアクセスするため順序非依存 |
