# Favnir Concurrency Draft

更新日: 2026-04-26

## 結論

Favnir では、次の 3 つを分けて考える方がよい。

- 非同期
- 並列
- コルーチン

この 3 つは似て見えるが、役割が違う。

初期仕様としては:

1. `async fn` / `async stage` / `async seq`
2. `Task<T>`（`Future<T>` ではなく `Task<T>` を採用。詳細は `favnir-async.md` 参照）

を先に固める。`await` キーワードは持たない。`bind` が `Task<T>` を自動解除する。

その後に:

3. 高レベル並列 API
4. `collect / yield`
5. `Stream<T>`
6. coroutine 的意味論

を段階的に足すのが自然。

## 1. 非同期

### 意味

非同期は「待ちを扱う」ための仕組み。

Favnir では:

- `async fn` / `async stage` / `async seq` — 宣言側
- `Task<T>` — 非同期の標準型（`await` キーワードなし）
- `bind` が `Task<T>` を自動解除（`async` スコープ内のみ）

で整理する。詳細は `favnir-async.md` を参照。

### 例

```fav
async fn main() -> Unit !Io {
    bind text <- IO.read_file("users.csv")  -- bind が Task<String> を解除
    IO.println(text)
}
```

### 位置づけ

- 外部 I/O
- network
- file
- db

のような待ちを伴う処理のための基本モデル。

## 2. 並列

### 意味

並列は「独立した処理を同時に進める」ための仕組み。

Favnir では low-level thread を直接見せるより、高レベル API で表した方がよい。

### 候補

- `Task.all` — 複数の Task を並列実行、全結果を待つ
- `Task.race` — 最初に完了した Task の結果を使う
- `Task.all` on list — `List.map` + `Task.all` で data parallel
- `parallel seq` — `seq` 内ステージを並列実行する宣言（将来）

### 例

```fav
-- 2つを並列実行
bind (a, b) <- Task.all(fetch_a(), fetch_b())

-- リストを並列処理
bind results <- users |> List.map(fetch_profile) |> Task.all
```

### 位置づけ

- 複数 I/O の同時実行
- data parallel な変換
- 独立 `stage` の分岐処理

### 方針

- 並列は必要
- ただし thread や lock を表面に出さない
- effectful composition として見せる

## 3. コルーチン

### 意味

コルーチンは「中断と再開ができる計算」。

欲しくなる場面:

- generator
- `yield`
- stream
- cooperative concurrency

### 価値

- データ分析とかなり相性が良い
- 列処理をきれいに書ける
- `collect / yield` とつながる

### 注意点

- runtime が一気に重くなる
- `async` と役割が混ざりやすい
- suspension point の設計が必要

### 方針

- 初期仕様には入れない
- まずは generator 的な限定機能から始める

## 4. `collect / yield`

コルーチンそのものを先に入れるより、まずは限定された列生成構文として `collect / yield` を考える方がよい。

例:

```fav
stage ActiveEmails: List<User> -> List<String> = |users| {
    collect {
        for user in users {
            if user.active {
                yield user.email
            }
        }
    }
}
```

### 意義

- 高階関数だけでは読みにくい処理を整理できる
- `Stream<T>` へ伸ばしやすい
- coroutine の前段階として導入しやすい

## 5. `Stream<T>`

もし遅延評価や generator 的処理を入れるなら、`Stream<T>` を間に置くのが自然。

### 方針

- 最初から全面 lazy にはしない
- 必要なら `Stream<T>` を stdlib 型として導入
- `collect / yield` とつなげる

### 例

```fav
stage ReadLines: String -> Stream<String> !Io = |path| {
    ...
}
```

## 6. Rust との違い

Rust では:

- async は強い
- 並列処理もある
- ただし言語中核として coroutine が強く前に出ているわけではない

Favnir では Rust をそのまま真似る必要はない。

むしろ:

- 非同期は `async/await`
- 並列は高レベル API
- coroutine は generator / stream 側から入る

という分離の方が自然。

## 7. 推奨導入順

1. `Task<T>` 型の定義
2. `async fn` / `async stage`（`bind` で自動解除、`await` キーワードなし）
3. `async seq`（非同期パイプライン）
4. `Task.run`（同期コンテキストからの実行）
5. `Task.all` / `Task.race`（並列実行）
6. `Task.timeout`（タイムアウト付き）
7. `collect / yield`
8. `Stream<T>`（遅延非同期シーケンス）
9. coroutine 的意味論

## 8. 重要な整理

Favnir では:

- 非同期 = 待ちを扱う
- 並列 = 独立処理を同時に進める
- コルーチン = 中断再開可能な計算を扱う

この 3 つを混ぜない方がよい。

## 短い結論

Favnir に並列処理はかなり合う。  
コルーチンも魅力はある。

ただし、初期仕様としては:

- `async/await`
- `Future<T>`

を先に固めるべき。

その後に、

- `all`
- `race`
- `par_map`
- `collect / yield`
- `Stream<T>`

を足していくのが現実的で、一貫性も高い。
