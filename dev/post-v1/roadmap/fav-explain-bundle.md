# fav explain + fav bundle 統合設計

日付: 2026-05-01

## 位置づけ

`fav explain` と `fav bundle` は別々のコマンドだが、**同じ基盤の上に乗る**。

- `fav bundle` — エントリポイントから到達可能なコードだけを `.fvc` にまとめる
- `fav explain` — プログラムの型・effect・依存構造を構造化出力する

共通の基盤は**到達可能性解析 (reachability analysis)** である。

```
source
  └─ reachability analysis (shared)
        ├─ fav bundle  → deployable .fvc + manifest.json
        ├─ fav explain → explain.json
        └─ fav graph   → graph.mmd / graph.json
```

この 3 コマンドは同じ解析パスから派生するため、
`fav bundle --explain` のような統合実行が自然に成立する。

---

## 1. fav bundle

### 役割

エントリポイントから実際に到達するコードだけを `.fvc` にまとめる。

到達不能な `fn / stage / seq` は含めない。
`rune` 依存は推移的に含める。

### CLI

```text
fav bundle src/main.fav
fav bundle src/main.fav -o dist/app.fvc
fav bundle src/main.fav -o dist/app.fvc --entry main
fav bundle src/main.fav -o dist/app.fvc --manifest
fav bundle src/main.fav -o dist/app.fvc --explain
```

デフォルト:

- エントリ: `main`
- 出力: `dist/<rune-name>.fvc`
- manifest: 出力しない（`--manifest` で有効化）
- explain: 出力しない（`--explain` で有効化）

### 出力ファイル

| フラグ | 生成ファイル |
|---|---|
| (なし) | `dist/app.fvc` |
| `--manifest` | `dist/app.fvc` + `dist/app.manifest.json` |
| `--explain` | `dist/app.fvc` + `dist/app.explain.json` |
| `--manifest --explain` | 両方 |

### manifest.json スキーマ

```json
{
  "version": "1.0",
  "entry": "main",
  "source": "src/main.fav",
  "artifact": "dist/app.fvc",
  "artifact_size": 4096,
  "built_at": "2026-05-01T00:00:00Z",
  "rune": {
    "name": "myapp",
    "version": "1.0.0"
  },
  "included": ["main", "ImportUsers", "ParseCsv", "ValidateUser", "SaveUsers"],
  "excluded": ["unused_helper"],
  "effects_required": ["Io", "Db"],
  "emits": ["UserCreated"],
  "runes_used": ["validate"]
}
```

`included` / `excluded` はデバッグと audit に直結する。
`effects_required` は実行環境の capability チェックに使える。

### fav bundle の保証

- `included` に含まれない関数は `.fvc` に存在しない
- `effects_required` は実行前の capability チェックに十分
- manifest は artifact と 1:1 対応し、再現性がある

---

## 2. fav explain

### 役割

ソースファイルまたは `.fvc` artifact から、型・effect・依存構造を構造化 JSON として出す。

「コードが何をするか」を機械可読な形で表現する。

LSP hover, Veltra explain pane, AI 補完, CI diff のすべてが同じ出力を読む。

### CLI

```text
fav explain src/main.fav
fav explain src/main.fav --format json
fav explain src/main.fav --format text
fav explain src/main.fav --entry main
fav explain src/main.fav --focus stage
fav explain dist/app.fvc
```

デフォルト:

- format: `text`（人間向け）
- `--format json`: 構造化 JSON（ツール向け）
- artifact からも explain を出せる（`fav explain dist/app.fvc`）

### explain.json スキーマ

```json
{
  "version": "1.0",
  "entry": "main",
  "source": "src/main.fav",
  "fns": [
    {
      "name": "main",
      "kind": "fn",
      "params": [],
      "return_type": "Unit",
      "effects": ["Io"],
      "calls": ["ImportUsers"],
      "reachable_from_entry": true
    }
  ],
  "stages": [
    {
      "name": "ParseCsv",
      "kind": "stage",
      "input_type": "String",
      "output_type": "List<UserRow>",
      "effects": [],
      "calls": [],
      "reachable_from_entry": true
    },
    {
      "name": "ValidateUser",
      "kind": "stage",
      "input_type": "UserRow",
      "output_type": "UserRow!",
      "effects": [],
      "calls": ["Flow.validate"],
      "reachable_from_entry": true
    },
    {
      "name": "SaveUsers",
      "kind": "stage",
      "input_type": "List<UserRow>",
      "output_type": "Int",
      "effects": ["Db"],
      "calls": ["Db.execute"],
      "reachable_from_entry": true
    }
  ],
  "seqs": [
    {
      "name": "ImportUsers",
      "kind": "seq",
      "input_type": "String",
      "output_type": "Int",
      "effects": ["Db"],
      "stages": ["ParseCsv", "ValidateUser", "SaveUsers"],
      "emits": [],
      "reachable_from_entry": true
    }
  ],
  "types": [
    {
      "name": "UserRow",
      "kind": "record",
      "fields": [
        { "name": "name", "type": "String" },
        { "name": "email", "type": "String" },
        { "name": "age", "type": "Int" }
      ]
    }
  ],
  "effects_used": ["Io", "Db"],
  "emits": [],
  "runes_used": ["validate"]
}
```

### --focus オプション

| フラグ | 出力範囲 |
|---|---|
| `--focus all` | fn + stage + seq + effects（デフォルト） |
| `--focus stage` | stage のみ |
| `--focus seq` | seq とその構成 stage |
| `--focus fn` | fn のみ |
| `--focus types` | type 定義のみ |

### text フォーマット（--format text）

```
fn main() -> Unit !Io
  calls: ImportUsers

seq ImportUsers: String -> Int !Db
  stages: ParseCsv -> ValidateUser -> SaveUsers

stage ParseCsv: String -> List<UserRow>
stage ValidateUser: UserRow -> UserRow!
stage SaveUsers: List<UserRow> -> Int !Db
  effects: Db
```

`fav explain` の text 出力は `fav explain` の代わりに人間が読む形。
JSON 出力はツールが読む形。両方が同じ解析から生成される。

---

## 3. 統合: fav bundle --explain

`--explain` フラグを付けると、bundle と explain を **一回の解析パスで**両方生成する。

```text
fav bundle src/main.fav -o dist/app.fvc --explain
```

生成物:

```
dist/
  app.fvc            ← deployable artifact
  app.manifest.json  ← bundle manifest (reachability + effects)
  app.explain.json   ← full explain output
```

### manifest と explain の関係

manifest は explain の **サブセット** である。

| フィールド | manifest | explain |
|---|---|---|
| `included` (関数名一覧) | あり | なし（詳細はexplainに） |
| `effects_required` | あり | あり（per-function詳細あり） |
| `emits` | あり | あり |
| 型情報 | なし | あり |
| per-function deps | なし | あり |
| `stages` (flw構成) | なし | あり |

manifest は軽量な「何が入っているか」の証明。
explain は「それぞれが何をするか」の詳細。

---

## 4. artifact からの explain

`.fvc` artifact に十分な metadata が含まれていれば、
ソースなしで explain を出せる。

```text
fav explain dist/app.fvc
fav explain dist/app.fvc --format json
```

これにより:

- デプロイ済み artifact の audit
- ソースがない環境での review
- CI での artifact diff

が成立する。

artifact explain の対応方針:

- `.fvc` v0x06 フォーマットに explain metadata セクションを追加する
- metadata がない古い artifact は `--format text` でスケルトンを出す

---

## 5. CI 連携: explain diff

explain JSON が安定したスキーマを持つことで、CI で diff が取れる。

```text
fav explain src/main.fav --format json > explain-new.json
diff explain-prev.json explain-new.json
```

あるいは専用コマンド（将来）:

```text
fav explain diff explain-prev.json explain-new.json
```

差分として検出したい変化:

- 新しい `effect` が追加された
- `emits` が変わった
- `seq` のステージ順が変わった
- 到達不能になった関数が増えた

これは PR review や artifact approval フローに直結する。

---

## 6. Veltra との統合

Veltra notebook は explain JSON を直接消費する。

| Veltra pane | 読むデータ |
|---|---|
| Explain pane | `app.explain.json` |
| Artifact pane | `app.manifest.json` (size, effects, runes) |
| Graph pane | explain.json から graph を生成（`fav graph` と共通） |

Veltra 側の実装は:

1. `fav bundle --explain` を実行して両方を得る
2. `explain.json` を explain pane にレンダリングする
3. `manifest.json` を artifact pane にレンダリングする
4. `explain.json` の `seqs.stages` から graph を描画する

Veltra は Favnir の explain 出力を **re-interpret しない**。
レンダリングするだけ。

---

## 7. LSP との統合

`fav lsp` の hover は、現在は型だけを出している。

explain JSON スキーマが固まると:

- hover → effect + calls + return type
- outline → seq/stage 一覧（explain.json の stages/seqs から）
- go-to-definition → explain.json の name → span マッピング

LSP は explain を「リアルタイム版」として持ち、
`fav explain --format json` は「スナップショット版」として持つ。

---

## 8. 実装順序

### Step 1: explain JSON スキーマの固定

まず JSON スキーマだけを決める。実装より先にスキーマを固定する。

対象:
- `fns` / `stages` / `seqs` / `types` の最小フィールド
- `effects_used` / `emits` / `runes_used`
- `reachable_from_entry` フラグ

### Step 2: fav explain --format json の実装

既存の `fav explain` (text出力) を JSON 出力に対応させる。

既存の IR + checker の情報で大半は出せる。

### Step 3: fav bundle の実装

reachability analysis を IR レベルで実装する。

`fav explain` の実装と共通のパスを使う。

manifest.json を生成する。

### Step 4: fav bundle --explain の統合

一回のパスで explain.json + manifest.json + .fvc を生成する。

### Step 5: artifact からの explain

`.fvc` フォーマットに explain metadata セクションを追加する。
`fav explain dist/app.fvc` を動作させる。

### Step 6: CI diff ツール

explain diff コマンドまたはスクリプト。

---

## 9. 一言でいうと

`fav bundle` は「何を含めるか」を決め、
`fav explain` は「それが何をするか」を出す。

同じ到達可能性解析の上に乗るため、
`fav bundle --explain` で一回のパスで両方が手に入る。

この出力は:

- Veltra pane のデータソース
- CI の artifact review 基盤
- LSP の metadata ソース
- AI 補完の静的 context

として機能する。
