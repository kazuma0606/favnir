# Favnir v5.1.0 〜 v6.0.0 ロードマップ

作成日: 2026-05-19

テーマ: **`rune` パッケージマネージャ + セルフホスト完成（開発一区切り）**

---

## 全体像

```
v5.0.0  (完了) AWS 本番稼働 + CI/CD + WASM リファレンスサイト
v5.1.0         セルフホスト前提条件（言語機能 + VM primitive 補完）
v5.2.0         パッケージ仕様 + Registry 拡張
v5.3.0         rune コマンド実装
v5.4.0         import rune → Favnir ソース解決
v5.5.0         公式 Rune publish
v5.6.0         依存解決 + rune.lock
v6.0.0         セルフホスト完成 ← 開発一区切り
```

---

## コマンド体系の原則

### ユーザー向けコマンド（`fav *` / `rune *` が起点）

```bash
# コンパイル・実行
fav run   source.fav          # コンパイル + 実行
fav build source.fav          # → source.fvc（FVC バイトコード）★既存実装済み
fav build source.fav -o out.fvc
fav run   source.fvc          # プリコンパイル済み .fvc を実行 ★既存実装済み
fav check source.fav          # 型チェックのみ

# スキーマ生成（既存）
fav build --graphql source.fav
fav build --proto   source.fav
fav build --schema  source.fav
fav build --wasm    source.fav

# パッケージ管理（v5.3.0〜）
rune install csv
rune publish
rune list
```

### 開発・デバッグ専用（ユーザーは使わない）

```bash
cargo build   # Rust VM のビルド（fav バイナリ生成）
cargo run     # 開発時のみ。fav * の内部実装として呼ばれる
cargo test    # Rust レベルのテスト
```

**目指す姿**: `cargo run` / `cargo build` は Favnir 開発者が VM を更新する際のみ使用。
エンドユーザー・CI・デプロイは全て `fav *` / `rune *` を起点とする。
Rust VM は `fav` バイナリの内部実装として隠蔽され、直接呼ばれない。

---

## アーキテクチャ原則: VM Primitive 境界線

**Rust に永続依存する層（自前実装しない）**:

| カテゴリ | VM Primitive | 理由 |
|---------|-------------|------|
| 暗号 | `Crypto.hmac_sha256_raw` `Crypto.sha256_raw` `Crypto.random_bytes_raw` `Crypto.base64_encode_raw` `Crypto.bcrypt_hash_raw` | セキュリティクリティカル。個人メンテ不可 |
| HTTP 通信 | `Http.send_raw` `Http.listen_raw` | TLS/TCP は `hyper`/`reqwest` に委ねる |
| ファイル I/O | `IO.read_file_raw` `IO.write_file_raw` `IO.write_bytes_raw` | OS インターフェース |
| データ形式 | `Csv.parse_raw` `Json.parse_raw` `Parquet.read_raw` | 各専用 crate に委ねる |
| クラウド SDK | `AWS.s3_put_raw` `AWS.dynamo_get_raw` 等 | SigV4 署名 + SDK |
| DB | `DuckDb.query_raw` `Db.query_raw` | DB ドライバ |
| VM 自体 | バイトコードインタプリタ | 常に Rust |

**Favnir で実装する層（v6.0.0 到達後）**:

```
┌─────────────────────────────────────────────┐
│  fav CLI ドライバ（fav run / fav check ...）  │  ← Favnir 実装
├─────────────────────────────────────────────┤
│  コンパイラ（パーサー + 型チェッカー + コード生成）│  ← Favnir 実装
├─────────────────────────────────────────────┤
│  標準 Rune（csv / http / auth / log ...）     │  ← Favnir 実装
├─────────────────────────────────────────────┤
│  VM Primitive（上表）                         │  ← 永続 Rust
└─────────────────────────────────────────────┘
```

---

## v5.1.0 — セルフホスト前提条件

コードベース調査（2026-05-19）で判明した不足機能を補完する。
v6.0.0 のセルフホスト実装を可能にするための基盤整備バージョン。

### A. 再帰的 sum type の動作確認・修正（Critical）

AST 定義に必須。以下が型チェックを通るよう確認・対応:

```favnir
type Expr =
  | Lit(Int)
  | Add(Expr, Expr)    // 直接自己参照
  | If(Expr, Expr, Expr)
  | Call(String, List<Expr>)
```

- `error_catalog.rs` の E0251「recursive type without indirection」と
  checker.rs の実装を照合し、sum type の再帰バリアントを許容するよう修正
- record type の直接再帰（`type Node = { next: Node }`）は引き続き E0251 でエラー
- sum type の再帰バリアントは VM が `Variant(String, Option<Box<VMValue>>)` で
  実行時に自然にハンドルするため実装上は問題ない

### B. ファイル I/O VM Primitive 追加（Critical）

コンパイラがソースファイルを読み書きするために必須:

```favnir
IO.read_file_raw(path: String) -> Result<String, String> !Io
IO.write_file_raw(path: String, content: String) -> Result<Unit, String> !Io
IO.write_bytes_raw(path: String, bytes: List<Int>) -> Result<Unit, String> !Io
IO.file_exists_raw(path: String) -> Bool !Io
```

### C. ビット演算 VM Primitive 追加（Critical）

バイトコード生成でバイト列を組み立てるために必須:

```favnir
Int.shl(x: Int, n: Int) -> Int    // 左シフト
Int.shr(x: Int, n: Int) -> Int    // 右シフト（符号なし）
Int.band(x: Int, y: Int) -> Int   // ビット AND
Int.bor(x: Int, y: Int) -> Int    // ビット OR
Int.bxor(x: Int, y: Int) -> Int   // ビット XOR
Int.bnot(x: Int) -> Int           // ビット NOT
Int.to_byte(x: Int) -> Int        // x & 0xFF（バイト正規化）
```

### D. バイトコード仕様書の作成・凍結（Critical）

Favnir コンパイラを Favnir で書く前に仕様を凍結する。
`docs/bytecode-spec.md` を作成し、以下を文書化:

- 全オペコード一覧（現在 22 個、番号固定）
- バイナリエンコーディング形式
- `FvcArtifact` のバイナリ構造
- バージョンフィールド・マジックナンバー
- 定数プールの形式

現在の全オペコード（`codegen.rs` より）:
```
0x01 Const          0x02 ConstUnit      0x03 ConstTrue     0x04 ConstFalse
0x10 LoadLocal      0x11 StoreLocal     0x12 LoadGlobal    0x13 Pop
0x14 Dup            0x15 Call           0x16 Return
0x20 Add            0x21 Sub            0x22 Mul           0x23 Div
0x24 Eq             0x25 Ne             0x26 Lt            0x27 Le
0x28 Gt             0x29 Ge             0x2A And           0x2B Or
0x30 Jump           0x31 JumpIfFalse    0x32 MatchFail     0x33 ChainCheck
0x34 JumpIfNotVariant
0x40 GetField       0x41 BuildRecord    0x42 MakeClosure   0x43 GetVariantPayload
0x50 CollectBegin   0x51 CollectEnd     0x52 YieldValue    0x53 EmitEvent
0x54 TrackLine
```

### E. `String.chars` 追加（Important）

レキサー実装で文字単位の処理を自然に書くために追加:

```favnir
String.chars(s: String) -> List<String>   // 各文字を単一文字の String として返す
```

（`Char` 型は新設せず、単一文字の String で代替）

---

## v5.2.0 — パッケージ仕様 + Registry 拡張

### Rune 個別パッケージの `rune.toml`

各 Rune ディレクトリ（例: `runes/csv/`）に追加:

```toml
[rune]
name        = "csv"
version     = "0.2.0"
description = "CSV parse/write with type-safe schema adaptation"
entry       = "csv.fav"
effects     = []

[dependencies]
# 他 Rune への依存（v5.6.0 で有効化）
```

### プロジェクトの `rune.toml`

プロジェクトルートに配置（`Cargo.toml` / `package.json` に相当）:

```toml
[project]
name    = "my-pipeline"
version = "0.1.0"
favnir  = ">=5.2.0"

[runes]
csv  = "0.2.0"
http = "1.0.0"
auth = "0.3.0"
```

インストール先: `./rune_modules/<name>/`（`.gitignore` 追加）

### Registry API 拡張

```
GET  /runes/{name}/download      ← zip を返す（新規）
GET  /runes/{name}/versions      ← バージョン一覧（新規）
POST /runes/{name}               ← zip blob を受け取るよう変更
```

S3 キー: `{name}/{version}.zip`（バージョン別保存、旧データ破棄 OK）

---

## v5.3.0 — `rune` コマンド実装

### コマンド体系

```bash
rune install csv           # rune.toml に追加 + DL → rune_modules/ 展開
rune install csv@0.2.0     # バージョン指定
rune install               # rune.toml の全 rune をインストール
rune uninstall csv
rune list                  # インストール済み一覧
rune info csv              # name / version / effects / description
rune search <query>        # registry 検索
rune update csv            # 最新バージョンに更新
rune publish               # カレントディレクトリを zip → registry
```

### `fav rune` との関係

- `fav rune install csv` でも `rune install csv` でも動く
- `fav` バイナリが `argv[0] == "rune"` の場合にパッケージマネージャモードで動作

---

## v5.4.0 — `import rune` ソース解決

### 現状 → 目標

```
# 現状
import rune "aws"  →  vm.rs の AWS ブロックを直接実行

# v5.4.0 目標
import rune "csv"  →  ./rune_modules/csv/csv.fav をコンパイル → 型解決 → 実行
import rune "aws"  →  ./rune_modules/aws/aws.fav
                        └─ aws.fav 内で AWS.s3_put_raw 等 VM primitive を呼ぶ
```

- 型チェッカー: `rune_modules/<name>/<entry>.fav` の public 型/関数を解決
- コンパイラ: Rune ソースを main プログラムとリンク
- フォールバック: `rune_modules/` になければ VM builtin にフォールバック（後方互換）

---

## v5.5.0 — 公式 Rune publish

既存 `runes/` の Favnir 実装を registry に公式 publish。各 Rune に `rune.toml` を追加。

| Rune | 優先度 | 依存 VM primitive |
|------|--------|-----------------|
| `csv` | ★★★ | `Csv.parse_raw` |
| `http` | ★★★ | `Http.send_raw` / `Http.listen_raw` |
| `auth` | ★★☆ | `Crypto.*` |
| `env` | ★★☆ | `Env.require_raw` |
| `log` | ★☆☆ | `IO.println` |
| `aws` | ★☆☆ | `AWS.*` |

---

## v5.6.0 — 依存解決 + rune.lock

- **`rune.lock`**: 全依存のバージョンをピン留め（再現性保証）
- **依存グラフ**: Rune が他 Rune に依存できる（`[dependencies]` を有効化）
- **バージョン競合検知**: 同一 Rune の異なるバージョン要求をアラート
- **`rune outdated`**: 更新可能な Rune を一覧表示
- **サイト更新**: Rune カタログページに install コマンドを追加

---

## v6.0.0 — セルフホスト完成

### ゴール

> `fav` バイナリ（Rust）は VM primitive の実行エンジンのみ。
> Favnir コンパイラ自身が Favnir で書かれ、自己コンパイルできる。

### ブートストラップ手順（最終形）

```
Stage 0: 現行 fav (Rust 製コンパイラ) ← ブートストラップ用に保持
Stage 1: fav_compiler.fav を Stage 0 でコンパイル → コンパイラバイトコード
Stage 2: Stage 1 のコンパイラで fav_compiler.fav を再コンパイル
Stage 3: Stage 1 == Stage 2 の出力 → ブートストラップ完了
```

### Phase A: Favnir でレキサー + パーサー実装

```favnir
import rune "fav-parse"
bind ast <- FavParse.parse(source_text);
```

- トークナイザーを Favnir で実装（`String.chars` + `IO.read_file_raw` を使用）
- AST 型を Favnir で定義（再帰的 sum type、v5.1.0 で対応済み）
- パーサーを Favnir で実装
- 既存 Rust パーサーの出力と一致することをテストで検証

### Phase B: Favnir で型チェッカー実装

```favnir
import rune "fav-check"
bind diagnostics <- FavCheck.check(ast);
```

- 型環境を `Map<String, String>` で表現
- 型推論・エフェクト検査を Favnir 関数で実装
- 既存 checker.rs の出力と一致することをテストで検証

### Phase C: Favnir でバイトコードコンパイラ実装

```favnir
import rune "fav-compile"
bind bytecode <- FavCompile.compile(ast);  // → List<Int>
```

- AST → バイトコード変換を Favnir で実装（`Int.shl` / `Int.band` 等を使用）
- バイトコード形式は `docs/bytecode-spec.md`（v5.1.0 で凍結）に従う
- `IO.write_bytes_raw` でファイルに書き出す

### Phase D: ブートストラップ検証

```bash
fav run fav_compiler.fav -- fav_compiler.fav
# → Stage 1 == Stage 2 を確認
```

- `fav` CLI ドライバを Favnir で実装
- ブートストラップ後: Rust コードは VM + primitive のみ
- 既存テスト全件 pass

---

## v6.0.0 完了条件

- [ ] `fav_compiler.fav` が Favnir で Favnir をコンパイルできる
- [ ] ブートストラップ検証: Stage 1 == Stage 2
- [ ] `rune install csv` → `import rune "csv"` が end-to-end で動く
- [ ] 公式 Rune が registry で配布されている
- [ ] `rune.lock` でプロジェクト依存が再現できる
- [ ] 既存テスト全件 pass
- [ ] `fav` Rust コードが VM + primitive のみ

---

## 実装メモ（随時更新）

- **Registry blob 変更**: S3 キーを `{name}/{version}.zip` に変更（旧データ破棄 OK）
- **`rune` バイナリ**: `argv[0] == "rune"` でパッケージマネージャモード
- **`rune_modules/` の gitignore**: 自動追加
- **再帰 sum type**: VM は `Variant(String, Option<Box<VMValue>>)` で実行時対応済み。checker 側の制約緩和のみ
- **`Int.band` 命名**: `Int.and` は論理 AND と混同するため `band`/`bor`/`bxor` に
