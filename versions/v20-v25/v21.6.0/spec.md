# v21.6.0 仕様書 — Playground v2（共有・テンプレート・ライブ統計）

## 概要

Favnir Playground をコードの「書き捨て」から「共有・発見・学習」の場へ昇格させる。

ロードマップ v21.6 の機能:
- 共有 URL（パーマリンク）
- フォーク（他人のコードを自分の環境にコピー）
- テンプレートギャラリー（よくあるパイプラインのサンプル集）
- 実行時間 / メモリ使用量の表示

**スコープ外（v21.6 では実装しない — ロードマップ v21.6 から意図的に延期）:**
- diff ビュー — v21.7 以降に延期。2 つのエディタ状態管理と Monaco diff API 連携が必要で単独バージョン相当の作業量
- `--format=flamegraph` のブラウザ表示 — v21.8 以降に延期。`fav profile` の flame データ形式確定と WASM 連携が必要

---

## アーキテクチャ

### URL 共有（2 段階）

#### ステージ 1: サーバーレス URL（クライアントサイドのみ）

コードを URL パラメータにエンコードして即座に共有できる。

```
https://play.favnir.dev/playground?c=<base64url-gzip>
```

- `TextEncoder` + `CompressionStream("gzip")` でコードを圧縮
- 結果を `base64url` でエンコード
- URL パラメータ `c` に格納
- ページロード時に `?c=` を読んで復元
- サーバー不要 — フォーク・共有が即時動作

#### ステージ 2: Lambda 短縮 URL（インフラ）

```
https://play.favnir.dev/playground?s=<slug>   (6文字 slug)
```

- `infra/share/` に Lambda + API Gateway を追加
- `POST /share { code: string }` → `{ slug: "abc123", url: "https://play.favnir.dev/playground?s=abc123" }`
- `GET /share/<slug>` → `{ code: string }`
- S3 バケット `favnir-playground-shares-<env>` にコードを保存（AES256 暗号化・Public Access Block 有効）
- TTL: 90日（S3 Lifecycle ルール）
- `playground?s=<slug>` パターンで Next.js の既存 `/playground` ルートが `?s=` を読んで Lambda から復元（専用 `/s/[slug]` ルートは不要）

### テンプレートギャラリー

Playground に "Templates" ドロップダウンを追加。
`site/lib/playground-templates.ts` に 6 テンプレートを定義。

### 実行時間 / メモリ表示

`performance.now()` で WASM 実行時間を計測し、出力パネルの下部に表示。
Chrome の `performance.memory` が利用可能な場合はメモリ使用量も表示。

---

## テンプレート一覧

| テンプレート名 | 説明 |
|---|---|
| `hello-world` | `IO.println` で Hello World |
| `pipeline-basic` | `stage` + `seq` + `|>` のシンプルなパイプライン |
| `list-transform` | `List.map` / `List.filter` / `List.fold` |
| `result-handling` | `Result<T, E>` + `?` 演算子 |
| `record-type` | レコード型 + フィールドアクセス |
| `fstring-format` | f-string + 文字列フォーマット |

---

## 変更ファイル一覧

### site（Next.js）

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `site/lib/share-url.ts` | 新規 | `encodeCode` / `decodeCode` ユーティリティ |
| `site/lib/playground-templates.ts` | 新規 | 6 テンプレート定義 |
| `site/app/playground/page.tsx` | 更新 | Share ボタン・テンプレートドロップダウン・実行統計 |
| `site/app/playground/share-api.ts` | 新規 | Lambda API クライアント（`shareCode` / `loadSharedCode`） |

### infra

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `infra/share/main.tf` | 新規 | Lambda + API Gateway + S3 バケット定義 |
| `infra/share/lambda.tf` | 新規 | Lambda 関数定義 |
| `infra/share/handlers/share.js` | 新規 | Lambda ハンドラ（POST/GET） |
| `infra/share/providers.tf` | 新規 | AWS provider |
| `infra/share/variables.tf` | 新規 | 変数定義 |
| `infra/share/outputs.tf` | 新規 | API Gateway URL 出力 |

### fav（Rust）

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | `version = "21.5.0"` → `"21.6.0"` |
| `fav/src/driver.rs` | 更新 | `v216000_tests` 追加 |

### CHANGELOG / docs

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `CHANGELOG.md` | 更新 | v21.6.0 エントリ追加 |
| `site/content/docs/tools/playground.mdx` | 新規 | Playground v2 ドキュメント |

---

## `share-url.ts` API

```typescript
// コードを base64url+gzip でエンコード → URL パラメータ用文字列
export async function encodeCode(code: string): Promise<string>

// URL パラメータ文字列をデコード → コード
export async function decodeCode(encoded: string): Promise<string>

// 現在のコードを ?c= パラメータ付き URL にする（encoded のみ受け取る）
export function buildShareUrl(encoded: string): string
```

### 実装方針（ブラウザ標準 API のみ）

```typescript
async function encodeCode(code: string): Promise<string> {
  const bytes = new TextEncoder().encode(code)
  const compressed = new CompressionStream('gzip')
  // ... WritableStream / ReadableStream
  return base64urlEncode(compressedBytes)
}
```

`CompressionStream` は Chrome 80+, Firefox 113+, Safari 16.4+ で対応。
未対応ブラウザは非圧縮の base64url にフォールバック。

---

## `playground-templates.ts` 型定義

```typescript
export interface PlaygroundTemplate {
  id: string
  name: string        // ドロップダウン表示名
  description: string // ツールチップ用
  code: string
}

export const PLAYGROUND_TEMPLATES: PlaygroundTemplate[]
```

---

## Lambda API 仕様

### `POST /share`

```json
// リクエスト
{ "code": "stage Double: Int -> Int = |n| n * 2\n..." }

// レスポンス 201
{ "slug": "abc123", "url": "https://play.favnir.dev/s/abc123" }
```

- slug: 6文字英数字（nanoid スタイル）
- コードは S3 の `shares/<slug>.fav` に保存
- 最大サイズ: 32KB

### `GET /share/<slug>`

```json
// レスポンス 200
{ "code": "stage Double: ..." }

// 404
{ "error": "not found" }
```

---

## Playground UI 変更

### Share ボタン（ヘッダー右）

```
[ テンプレート ▼ ]  [ 型チェック ]  [ ▶ 実行 ]  [ 📋 共有 ]
```

「共有」ボタンの動作:
1. `encodeCode(code)` でサーバーレス URL を生成
2. クリップボードにコピー
3. ボタンラベルを「✓ コピー済み」に 2 秒間変更
4. （Lambda API 利用可能な場合）短縮 URL を優先

### URL からの復元（フォーク）

- ページロード時に `?c=` パラメータを検出
- デコードしてエディタに設定
- バナー表示: `「共有コードを読み込みました — 自由に編集できます」`

### テンプレートドロップダウン

- 「テンプレート ▼」ボタンを押すと 6 テンプレートの選択 UI が開く
- 選択するとエディタのコードを置き換え（確認なし）

### 実行統計（出力パネル下部）

```
実行時間: 12ms  |  メモリ: 1.2MB（推定）
```

- `performance.now()` で WASM instantiate 〜 main() 呼び出し完了まで計測
- `performance.memory.usedJSHeapSize` で JS ヒープ使用量（Chrome のみ）
- 出力が `(出力なし)` の場合も統計は表示

---

## 完了条件

- [ ] `?c=` パラメータ経由でコードが正確に復元できる
- [ ] エンコード→デコードのラウンドトリップが 1000 文字のコードで正確
- [ ] テンプレート 6 種が選択可能でエディタに反映される
- [ ] 実行時間が出力パネルに表示される
- [ ] Lambda share API: `POST /share` → slug → `GET /share/<slug>` でコードが復元できる（ローカルスタック / AWS 双方で確認）
- [ ] `cargo test v216000` — 8/8 PASS
- [ ] `cargo test` — リグレッションなし（1817 件以上合格）
- [ ] `CHANGELOG.md` に v21.6.0 エントリ
- [ ] `site/content/docs/tools/playground.mdx` 作成済み
