# Favnir Async Design: Task<T>

更新日: 2026-05-03

`async` は v1.0.0 スコープ外。post-v1 (v2.0.0) で実装する。

---

## 概要

Favnir の非同期モデルは、`await` キーワードを持たない。
代わりに **`Task<T>`** という標準型と、既存の `bind` / `chain` 語彙を拡張することで、
非同期処理を「もう一つの effect」として自然に扱う。

> **設計原則**:
> `bind` はすでに「値を取り出して続ける」という意味を持つ。
> `async fn/stage` の内部では、`bind` が `Task<T>` を自動的に解除する。
> `await` という別キーワードは不要。

---

## 1. `Task<T>`: 非同期の標準型

```
Task<T> = 「いつか T を生成する計算（工程）」
```

`Task<T>` は**値**であり、宣言しただけでは実行されない。
`async fn main` のランタイムか、`Task.run` によって初めて実行される。

`Future<T>`（Rust/Python）との違い:
- Favnir では「Stage（工程）・Sequence（順序）」が中心語彙であり、
  「Task（仕事）」はその延長として直感的に一致する。
- 「Future = すでに始まっている」に対し、「Task = これから実行する工程」という語感が Favnir に合う。

---

## 2. `async` 宣言

`async` を `fn` / `stage` に付けることで、その返り値が `Task<T>` に変わる。

```fav
-- 通常の stage: Url -> String !Network
async stage FetchText: Url -> String !Network = |url| {
    ...
}
-- 外から見た型: Url -> Task<String> !Network
```

シグネチャは `String` と書くが、呼び出し元には `Task<String>` として見える。
これにより「この関数は非同期だ」という情報がシグネチャに現れる。

`async fn` も同様:

```fav
async fn main() -> Unit !Io !Network {
    ...
}
```

---

## 3. `bind` による自動解除（暗黙 await）

**`async fn/stage` の内部**では、`bind` が `Task<T>` を自動的に解除する。
`await` キーワードは不要。

```fav
async stage ParseAndSave: Url -> Int !Network !Db = |url| {
    bind text  <- FetchText(url)    -- Task<String> を bind が解除 → text: String
    bind rows  <- Csv.parse(text)   -- 同期: そのまま解除 → rows: List<Row>
    bind count <- Db.insert(rows)   -- 同期 !Db: そのまま → count: Int
    count
}
```

**`async` でないスコープ**では、`bind` は `Task<T>` を解除しない（型エラーになる）。
非同期を明示したい場所でしか非同期コードは書けない。

---

## 4. `chain` との連携（Task + Result）

`chain` は `Task<T>!` から `T` を取り出す（Task 解除 + Result 解除を同時に行う）。

```fav
async stage SafeFetch: Url -> String !Network = |url| {
    chain body <- IO.http_get(url)   -- Task<String>! → String (失敗なら伝播)
    body
}
-- 外から見た型: Url -> Task<String>! !Network
```

| 操作 | `bind` | `chain` |
|---|---|---|
| `Task<T>` の解除 | ✓ | ✓ |
| `Result<T, E>` / `Option<T>` の伝播 | ✗ | ✓ |
| `Task<T>!` の一括処理 | ✗ | ✓ |

---

## 5. `async seq`: 非同期パイプライン

`async seq` を宣言すると、パイプライン全体が `Task` コンテキストで実行される。
純粋な `stage` は自動的に Task にリフトされる。

```fav
async seq FetchAndProcess =
    FetchText       -- Url -> Task<String> !Network  (async stage)
    |> ParseCsv     -- String -> List<Row>            (純粋、自動リフト)
    |> ValidateRow  -- Row -> Row!                    (純粋、自動リフト)
    |> SaveRows     -- List<Row> -> Int !Db           (同期 !Db、自動リフト)

-- 推論された型: Url -> Task<Int> !Network !Db
```

---

## 6. Task の実行境界

`Task<T>` は宣言しただけでは実行されない。
実行するには以下の方法を使う。

### `async fn main`（推奨）

ランタイムが `main` を Task として実行する。内部で `bind` が使える。

```fav
async fn main() -> Unit !Io !Network !Db {
    bind count <- FetchAndProcess(target_url)  -- bind が Task を解除
    IO.println_int(count)
}
```

### `Task.run`（同期コンテキストからの明示実行）

同期スコープから Task を実行する。完了までブロックする。

```fav
fn sync_entry() -> Unit !Io {
    bind result <- Task.run(FetchAndProcess(url))
    IO.println_int(result)
}
```

---

## 7. 並列実行 API

```fav
-- 複数の Task を並列実行し、全結果を待つ
bind (a, b) <- Task.all(FetchText(url1), FetchText(url2))

-- リスト版
bind results <- urls |> List.map(FetchText) |> Task.all

-- 最初に完了した Task の結果を使う（残りはキャンセル）
bind first <- Task.race(FetchText(url1), FetchText(url2))

-- タイムアウト付き
bind result <- FetchText(url) |> Task.timeout(5.0)
-- 型: Task<String>! !Network （タイムアウトは Err として伝播）
```

---

## 8. Effect システムとの統合

`async` は execution model（いつ実行されるか）であり、
`!Network` / `!Db` / `!Io` は effect（何に触れるか）。
この 2 つは独立して記述する。

```fav
async stage FetchAndStore: Url -> Int !Network !Db = |url| {
    chain body  <- IO.http_get(url)   -- !Network
    chain rows  <- Csv.parse(body)    -- Pure
    bind  count <- Db.insert(rows)    -- !Db
    count
}
```

`fav explain` の出力:

```
async stage FetchAndStore
  input:   Url
  output:  Task<Int>!
  effects: Network, Db
```

---

## 9. `await` キーワードを持たない理由まとめ

| 観点 | `await` あり | Favnir (`bind` 自動解除) |
|---|---|---|
| 学習コスト | 新キーワードが必要 | `bind` の意味を拡張するだけ |
| 明示性 | `await` の有無で非同期が見える | `async` 宣言で境界が見える |
| 一貫性 | `bind` と `await` が共存し冗長 | `bind` だけで统一 |
| AI フレンドリー | `await` を付け忘れるリスク | `async` スコープ内で自動 |
| effect との関係 | 別次元の概念として混在 | effect と対等に管理 |

---

## 10. 将来の拡張候補

- **`Task<T>?`**: キャンセル可能な Task（`Option<T>` との合成）
- **`Stream<T>`**: 複数値を非同期に生成するシーケンス（`Task` の多値版）
- **`par_map`**: データ並列（`List.map` の非同期版）
- **Structured concurrency**: Task のスコープを lexical に管理する仕組み

---

## 一言でいうと

> `async` は「この工程（Stage）は待つ」という宣言。
> `bind` は「待って、値を取り出す」という実行。
> `Task<T>` は「まだ完成していない工程の結果」。
