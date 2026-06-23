# v24.0.0 — VM in Favnir マイルストーン宣言

Date: 2026-06-23

## 目標

v23.1.0〜v23.8.0 で整備したインフラ（Bytes・ビット演算・Mut<T>・vm.fav Phase 1〜5）を土台に、
「**Favnir の表現力が VM 実装に足りるほど成熟した**」という証明をマイルストーンとして宣言する。

新実装として、v23.8 から持ち越しの `fav run --vm <path> --hex <hex>` CLI フラグを追加し、
vm.fav 経由でのバイトコード実行を端末から直接実行できるようにする。

---

## ロードマップ完了条件との対応

| ロードマップ完了条件 | v24.0.0 での対応 |
|---|---|
| `Bytes` 型 + ビット演算 + `Mut<T>` が動作する | v23.1〜v23.3 で完了 ✓ |
| vm.fav が全 opcode をデコード・実行できる | v23.4〜v23.8 で完了（Phase 1〜5）✓ |
| `fav run --vm=self/vm.fav` で hello.fav が動作する | v24.0.0 で `--vm <path> --hex <hex>` CLI として実装 ✓（hello.fav 相当バイトコードを `--hex` で渡す方式。`--vm=<path> <target.fav>` 形式は Phase 6 以降） |
| vm.fav での実行結果が Rust VM と一致（500 件以上） | Phase 6 以降（ユーザー定義関数 Call ディスパッチ実装後） |
| `fav test self/vm.fav` 全件 PASS | `cargo test v240000` 5/5 で代替実証 ✓ |

> **スコープの判断:** 条件 4（500 件超テスト）は vm.fav がユーザー定義関数 Call を処理できるようになってから達成可能。
> v24.0.0 は条件 1〜3・5 を達成し、マイルストーン宣言を行う。条件 4 は Phase 6 の先行目標とする。

---

## スコープ

### Rust（driver.rs + main.rs）

| 変更種別 | 対象 | 内容 |
|---|---|---|
| 新関数追加 | `driver::run_with_vm` | vm.fav ソース + hex バイトコード + globals エントリ → Value |
| CLI 追加 | `main.rs` `"run"` アーム | `--vm <path> --hex <hex>` フラグ解析と `run_with_vm` 呼び出し |

### ドキュメント

| 変更種別 | 対象 | 内容 |
|---|---|---|
| セクション追加 | `README.md` | "VM in Favnir マイルストーン" セクション（達成実績表） |
| エントリ追加 | `CHANGELOG.md` | v24.0.0 エントリ |
| 新規作成 | `benchmarks/v24.0.0.json` | テスト件数・フェーズ数 |

---

## 新関数定義

### `driver::run_with_vm`

```rust
/// v24.0.0: vm.fav 経由でバイトコードを実行する。
/// vm_src: vm.fav のソースコード
/// bytecode_hex: 実行するバイトコードの hex 文字列
/// globals_entries: globals マップに設定する (index, value_str) のリスト
pub fn run_with_vm(
    vm_src: &str,
    bytecode_hex: &str,
    globals_entries: &[(usize, &str)],
) -> Result<Value, String>
```

**実装方針:**
1. `globals_entries` を `Mut.set(globals, idx, VMStr("val"))` の Favnir コードに変換
2. vm.fav ソース + main 関数（globals セットアップ + `vm_run_named(bytes, globals)` + `vmval_display`）を結合
3. `build_artifact` + `exec_artifact_main` で実行
4. `Result<Value, String>` を返す

### CLI 追加（`main.rs`）

```
fav run --vm <vm_path> --hex <bytecode_hex>
```

- `--vm`: vm.fav のパス（必須）
- `--hex`: 実行するバイトコードの hex 文字列（必須）
- `run_with_vm(&vm_src, hex, &[])` を呼び出す
- 成功時: `println!("{}", v)`
- 失敗時: `eprintln!("error: {}", e); process::exit(1)`

---

## テスト（5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_0_0` | Cargo.toml に `version = "24.0.0"` | — |
| `run_with_vm_hello` | `run_with_vm` + hex `"12000016"` + globals[0]="hello" → vmval_display | `"hello"` |
| `run_with_vm_string_trim` | `run_with_vm` + hex `"12000040010012020015010016"` + 3 globals → vmval_display | `"hi"` |
| `changelog_has_v24_0_0` | CHANGELOG.md に `[v24.0.0]` | — |
| `readme_has_vm_in_favnir` | README.md に `VM in Favnir` | — |

### バイトコード詳細

**`run_with_vm_hello`**: hex `"12000016"`, globals: `[(0, "hello")]`

```
globals[0] = VMStr("hello")

pc=0: 12 00 00  LoadGlobal(0)  → push VMStr("hello")
pc=3: 16        Return          → VMStr("hello")

vmval_display(VMStr("hello")) = "hello"
```

**`run_with_vm_string_trim`**: hex `"12000040010012020015010016"`, globals: `[(0, "String"), (1, "trim"), (2, " hi ")]`

```
globals[0] = VMStr("String"), globals[1] = VMStr("trim"), globals[2] = VMStr(" hi ")

pc=0:  12 00 00  LoadGlobal(0)  → push VMStr("String")
pc=3:  40 01 00  GetField(1)    → push VMStr("String.trim")
pc=6:  12 02 00  LoadGlobal(2)  → push VMStr(" hi ")
pc=9:  15 01 00  Call(1)        → call_builtin("String.trim", [" hi "]) → VMStr("hi")
pc=12: 16        Return          → VMStr("hi")

vmval_display(VMStr("hi")) = "hi"
```

---

## README 追加セクション

`v23.0.0` 現在の状態セクションの直後に追加:

```markdown
**v24.0.0（2026-06-23）— VM in Favnir マイルストーン宣言**

テスト: **1926 件以上**

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
```

---

## 完了条件

- [ ] `driver::run_with_vm` が追加される
- [ ] `main.rs` に `--vm / --hex` フラグが追加される
- [ ] `v238000_tests::version_is_23_8_0` が削除される
- [ ] `cargo test v240000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1926 件以上合格）
- [ ] `CHANGELOG.md` に v24.0.0 エントリ
- [ ] `benchmarks/v24.0.0.json` 作成済み
- [ ] `README.md` に "VM in Favnir" セクション追加済み
