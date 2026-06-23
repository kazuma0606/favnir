# v20.6.0 Spec — io_uring 非同期 I/O（Linux）

## 概要

v20.6.0 は Linux 5.1+ 環境における **I/O サブシステムの根本的改善**を実装する。
`io_uring` カーネルインターフェースを `tokio-uring` クレート経由で利用し、
複数ファイルの並列読み込みを従来の `epoll` / `read()` 方式から
ゼロコンテキストスイッチの非同期 I/O に切り替える。

新プリミティブ `IO.read_files_batch(paths: List<String>) -> List<String>` を追加する。
Linux では `tokio-uring` による io_uring バックエンドを使用し、
Windows / macOS では `rayon` による並列 `read_to_string` にフォールバックする。
WASM では `err_vm` を返す。

**テーマ**: Runtime Excellence シリーズ第6弾 — Linux 本番環境の I/O 最適化

---

## 動機と期待効果

### 現状の問題

```
現状（epoll / 逐次 read）:
  paths.iter().map(|p| std::fs::read_to_string(p))
  → 各ファイルごとに open() + read() syscall
  → 1 ファイル = 2 回のコンテキストスイッチ（カーネル↔ユーザー空間）
  → 1000 ファイル = 2000 回のコンテキストスイッチ
```

### 最適化後（Linux io_uring）

```
io_uring バックエンド:
  ring buffer に 1000 件の read リクエストを一括 submit
  → カーネルが非同期でファイルを読み込み、完了を ring buffer に書き込む
  → ユーザー空間はポーリングで完了イベントを取得
  → コンテキストスイッチ ≈ 0 回（理想値）
```

### 期待改善（v20.5.0 比）

| ベンチマーク | v20.5.0 基準 | 期待改善 |
|---|---|---|
| `io_batch_100_files_ms` | ~85ms（逐次 read） | **+2〜4x**（io_uring 並列） |
| `io_db_file_mixed_ms` | ~240ms（DB + ファイル混在） | **+1.5〜2x** |
| `io_batch_1000_files_ms` | ~820ms | **+3〜5x** |

---

## アーキテクチャ

### プラットフォーム分岐

```
IO.read_files_batch(paths: List<String>) -> List<String>
      │
      ├─ Linux (cfg(target_os = "linux"))
      │      tokio_uring::start(async {
      │          join_all(paths.iter().map(read_one_uring)).await
      │      })
      │
      ├─ Windows / macOS (cfg(not(target_os = "linux")), not(wasm32))
      │      paths.par_iter()                  // rayon 並列（既存依存）
      │          .map(|p| std::fs::read_to_string(p))
      │          .collect::<Result<Vec<_>, _>>()
      │
      └─ WASM
             err_vm("IO.read_files_batch: not supported on wasm32")
```

### `tokio-uring` の依存宣言

```toml
# Cargo.toml — Linux 専用セクションを新規追加
[target.'cfg(target_os = "linux")'.dependencies]
tokio-uring = "0.4"
```

> **注意**: `tokio-uring` は Linux カーネル 5.1+ が必要。
> カーネル 5.1 未満の Linux では実行時エラーになるため、
> `execute_duckdb_pushdown` 同様 `Err(...)` を返してフォールバックする。
> ただし Phase 1 では実行時カーネルバージョン検出は実装しない（Err として記録するのみ）。

---

## 新プリミティブ: `IO.read_files_batch`

```favnir
// パイプライン例
stage LoadShards: List<String> -> List<String> = |paths| {
  IO.read_files_batch(paths)    // Linux: io_uring、その他: rayon fallback
}
```

### シグネチャ

```
IO.read_files_batch(paths: List<String>) -> List<String>
```

- 入力: ファイルパスのリスト
- 出力: ファイル内容のリスト（入力と同じ順序）
- いずれかのファイル読み込みが失敗した場合: `err_vm(VMValue::Str(error_message))`
- 全成功の場合: `ok_vm(VMValue::List([content1, content2, ...]))`

---

## `read_files_batch_impl` 実装詳細

### Linux バックエンド

```rust
#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
pub(crate) fn read_files_batch_impl(paths: &[String]) -> Result<Vec<String>, String> {
    tokio_uring::start(async {
        let handles: Vec<_> = paths.iter()
            .map(|p| read_one_uring(p.clone()))
            .collect();
        futures::future::try_join_all(handles).await
    })
    .map_err(|e| format!("IO.read_files_batch (io_uring): {e}"))
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
async fn read_one_uring(path: String) -> Result<String, String> {
    use tokio_uring::fs::File;

    let size = std::fs::metadata(&path)
        .map_err(|e| format!("IO.read_files_batch: metadata '{}': {e}", path))?
        .len() as usize;

    if size == 0 {
        return Ok(String::new());
    }

    let file = File::open(&path).await
        .map_err(|e| format!("IO.read_files_batch: cannot open '{}': {e}", path))?;

    let buf = vec![0u8; size];
    let (res, mut buf) = file.read_at(buf, 0).await;
    let bytes_read = res.map_err(|e| format!("IO.read_files_batch: read '{}': {e}", path))?;
    buf.truncate(bytes_read);

    String::from_utf8(buf)
        .map_err(|e| format!("IO.read_files_batch: utf8 '{}': {e}", path))
}
```

> **実装注意**: `tokio-uring 0.4` の API は実装前に確認が必要。
> 特に `read_at` のシグネチャ（`(buf, offset)` → `(Result, buf)` のムーブセマンティクス）と
> `File::open` の戻り型を確認すること:
> ```bash
> grep -rn "pub async fn open\|pub async fn read_at\|pub fn start" \
>   ~/.cargo/registry/src/*/tokio-uring-0.4*/src/ 2>/dev/null | head -10
> ```

### 非 Linux バックエンド（rayon 並列）

```rust
#[cfg(all(not(target_os = "linux"), not(target_arch = "wasm32")))]
pub(crate) fn read_files_batch_impl(paths: &[String]) -> Result<Vec<String>, String> {
    use rayon::prelude::*;
    paths.par_iter()
        .map(|p| std::fs::read_to_string(p)
            .map_err(|e| format!("IO.read_files_batch: cannot read '{}': {e}", p)))
        .collect()
}
```

### vm_call_builtin への追加

```rust
// vm_call_builtin 内、IO セクションに追加

"IO.read_files_batch" => {
    let paths = match args.into_iter().next() {
        Some(VMValue::List(list)) => {
            list.iter().map(|v| match v {
                VMValue::Str(s) => Ok(s.clone()),
                other => Err(format!(
                    "IO.read_files_batch: path must be String, got {}",
                    vmvalue_type_name(other)
                )),
            }).collect::<Result<Vec<_>, _>>()?
        }
        _ => return Err("IO.read_files_batch: expected List<String>".to_string()),
    };
    // cfg 分岐: Linux / 非Linux / WASM
    #[cfg(not(target_arch = "wasm32"))]
    {
        match read_files_batch_impl(&paths) {
            Ok(contents) => {
                let list = FavList::new(contents.into_iter()
                    .map(VMValue::Str)
                    .collect::<Vec<_>>());
                Ok(ok_vm(VMValue::List(list)))
            }
            Err(e) => Ok(err_vm(VMValue::Str(e))),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = paths;
        Ok(err_vm(VMValue::Str(
            "IO.read_files_batch: not supported on wasm32".to_string(),
        )))
    }
}
```

> **型変換注意**: `FavList` は `FromIterator` を実装していないため `collect::<FavList>()` は使えない。
> `FavList::new(vec_of_vmvalues)` を使う（vm.rs の既存パターンと同様）。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | `[target.'cfg(target_os = "linux")'.dependencies]` セクション新規追加 + `tokio-uring = "0.4"` / `futures = "0.3"`（未存在の場合 native-only deps に追加）/ version `20.5.0` → `20.6.0` |
| `fav/src/backend/vm.rs` | `read_files_batch_impl`（Linux / 非Linux / cfg 分岐） + `read_one_uring`（Linux async） + `"IO.read_files_batch"` primitive（`vm_call_builtin`） |
| `fav/src/driver.rs` | `v206000_tests`（5 件） |
| `CHANGELOG.md` | v20.6.0 エントリ追加 |
| `benchmarks/v20.6.0.json` | 実測ベンチマーク結果 |
| `site/content/docs/runes/io.mdx` | `IO.read_files_batch` ドキュメント追加 |

---

## テスト（v206000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_6_0` | `Cargo.toml` に `"20.6.0"` が含まれる |
| `io_batch_reads_single_file` | 1 ファイルを `read_files_batch_impl` で読み込み、内容が一致する |
| `io_batch_reads_multiple_files` | 3 ファイルを並列読み込み、全内容が正しい順序で返る |
| `io_batch_preserves_order` | 入力パスの順序通りに結果が返ることを確認 |
| `io_batch_error_on_missing_file` | 存在しないパスを含む場合に `Err` が返る |

### テスト用ヘルパー

```rust
// driver.rs v206000_tests 内

macro_rules! temp_txt {
    ($content:expr) => {{
        let mut p = std::env::temp_dir();
        p.push(format!("fav_v206_{}_{}.txt", std::process::id(), line!()));
        std::fs::write(&p, $content).unwrap();
        p
    }};
}
```

---

## 完了条件

- [ ] `IO.read_files_batch(paths)` が Linux / 非Linux 両環境で動作する
- [ ] Linux では `tokio-uring` が使われる（`cfg(target_os = "linux")` ガード）
- [ ] 非 Linux（Windows / macOS）では `rayon` フォールバックが動作する
- [ ] WASM ビルドが `cfg(target_arch = "wasm32")` ガードでコンパイルを通る
- [ ] `cargo test v206000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `fav/Cargo.toml` version が `20.6.0`
- [ ] `CHANGELOG.md` に v20.6.0 エントリが追加されている
- [ ] `benchmarks/v20.6.0.json` が生成されている
- [ ] `io_batch_100_files_ms` が v20.5.0 比 +2x 以上改善（Linux 本番環境）

---

## 技術ノート

### `tokio-uring` 0.4 の注意点

`tokio-uring` はデフォルトの `tokio` ランタイムとは**別の**ランタイムを起動する。
`tokio_uring::start(async { ... })` はブロッキング呼び出しとして動作するため、
既存の `tokio::spawn` や `tokio::runtime::Handle::block_on` とは混在できない。
`read_files_batch_impl` はスレッドプールの外側で呼ばれる同期関数として実装し、
`tokio_uring::start` でラップする。

> **確認事項**: 実装前に `grep -n "tokio::main\|block_on" fav/src/main.rs` で
> `vm_call_builtin` を呼ぶパスが `#[tokio::main]` や `block_on` の内側にないことを確認する。

```rust
// 正しい使い方:
tokio_uring::start(async {
    // ここは io_uring ランタイムの中
    let f = tokio_uring::fs::File::open(path).await?;
    // ...
})
```

### `read_at` のムーブセマンティクス

`tokio-uring 0.4` では `read_at` の戻り値が `(Result<usize, io::Error>, Vec<u8>)` の形。
バッファは所有権で渡され、読み込み後に返ってくる（GC なし）。

```rust
let buf = vec![0u8; file_size];
let (result, buf) = file.read_at(buf, 0).await;
result?;   // エラーチェック
// buf にデータが入っている
```

### `rayon::prelude::ParallelIterator` の import

非 Linux フォールバックで `par_iter()` を使う。`rayon = "1"` は既存 native-only 依存のため
追加不要。`use rayon::prelude::*;` を関数ローカルに追加するだけでよい。

### `FavList` への変換

`VMValue::List` の中身は `FavList` 型。`FavList` は `FromIterator` を実装していないため
`collect::<FavList>()` は使えない。確認済みの唯一の構築パターン:

```rust
// vm.rs の標準パターン（FavList::new が唯一の構築手段）:
let list = FavList::new(contents.into_iter().map(VMValue::Str).collect::<Vec<_>>());
Ok(ok_vm(VMValue::List(list)))
```

実装前に `grep -n "FavList::new" fav/src/backend/vm.rs | head -10` で既存パターンを確認する。

### スコープ外（v20.7 以降）

- io_uring を使った書き込み（`IO.write_files_batch`）
- io_uring + mmap の組み合わせ（直接バッファ登録）
- カーネルバージョン 5.1 未満の実行時検出と自動フォールバック
- `io_uring` のリングバッファサイズ設定（現状はデフォルト値を使用）
