export interface PlaygroundTemplate {
  id: string
  name: string
  description: string
  code: string
}

export const PLAYGROUND_TEMPLATES: PlaygroundTemplate[] = [
  {
    id: 'hello-world',
    name: 'Hello World',
    description: 'IO.println で文字列を出力',
    code: `public fn main() -> Unit !Io {
  IO.println("Hello, Favnir!")
}`,
  },
  {
    id: 'pipeline-basic',
    name: 'パイプライン基礎',
    description: 'stage + seq + |> の組み合わせ',
    code: `stage Double: Int -> Int = |n| n * 2
stage AddOne: Int -> Int = |n| n + 1

seq Transform = Double |> AddOne

public fn main() -> Unit !Io {
  IO.println_int(Transform(5))
}`,
  },
  {
    id: 'list-transform',
    name: 'List 操作',
    description: 'map / filter / fold の活用',
    code: `public fn main() -> Unit !Io {
  bind nums <- [1, 2, 3, 4, 5]
  bind evens <- List.filter(nums, |n| n % 2 == 0)
  bind doubled <- List.map(evens, |n| n * 2)
  bind total <- List.fold(doubled, 0, |acc, n| acc + n)
  IO.println_int(total)
}`,
  },
  {
    id: 'result-handling',
    name: 'Result 型',
    description: 'Result<T, E> + ? 演算子によるエラーハンドリング',
    code: `fn parse_positive(s: String) -> Result<Int, String> =
  match Int.parse(s) {
    Some(n) => if n > 0 { Result.ok(n) } else { Result.err("not positive") }
    None    => Result.err(f"parse error: {s}")
  }

public fn main() -> Unit !Io {
  match parse_positive("42") {
    Result.ok(n)  => IO.println_int(n)
    Result.err(e) => IO.println(e)
  }
}`,
  },
  {
    id: 'record-type',
    name: 'レコード型',
    description: 'type 定義 + フィールドアクセス',
    code: `type User = {
  name: String
  age: Int
}

fn greet(u: User) -> String =
  f"Hello, {u.name}! Age: {u.age}"

public fn main() -> Unit !Io {
  bind user <- User {
    name: "Alice"
    age: 30
  }
  IO.println(greet(user))
}`,
  },
  {
    id: 'fstring-format',
    name: 'f-string フォーマット',
    description: 'f-string による文字列補間',
    code: `public fn main() -> Unit !Io {
  bind name <- "Favnir"
  bind version <- 21
  IO.println(f"Welcome to {name} v{version}!")
}`,
  },
]
