# v20.6.0 実装計画 — io_uring 非同期 I/O（Linux）

## 実装順序

```
T1: Cargo.toml — tokio-uring 追加（Linux 専用セクション）         ← 最初（API 確認含む）
T2: vm.rs — read_files_batch_impl（Linux / 非Linux）+ IO.read_files_batch primitive ← T1 完了後
T3: driver.rs — v206000_tests（5 件）                               ← T2 完了後
T4: Cargo.toml version bump（20.5.0 → 20.6.0）                    ← 任意
T5: CHANGELOG.md 更新 + benchmarks/v20.6.0.json                   ← T3 完了後
T6: site/content/docs/runes/io.mdx 更新                           ← T5 完了後
```

**変更ファイル一覧:**
- `fav/Cargo.toml`（T1 / T4）
- `fav/src/backend/vm.rs`（T2）
- `fav/src/driver.rs`（T3）
- `CHANGELOG.md`（T5）
- `benchmarks/v20.6.0.json`（T5）
- `site/content/docs/runes/io.mdx`（T6）

---

## T1: `fav/Cargo.toml` — `tokio-uring` 追加

### 変更点

```toml
# 既存の native-only セクションとは別に、Linux 専用セクションを新規追加
[target.'cfg(target_os = "linux")'.dependencies]
tokio-uring = "0.4"
```

> **注意**: `tokio-uring` は既存の `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`
> セクションには追加しない。Linux 以外の native ビルド（Windows / macOS）では
> `rayon` フォールバックを使うため、`tokio-uring` は不要。

### 事前確認: `tokio-uring 0.4` API 確認

T1 完了後（`cargo update` で `Cargo.lock` に追加された後）、以下を確認する:

```bash
# tokio-uring 0.4.x がダウンロードされたか確認
grep "tokio-uring" fav/Cargo.lock | head -3

# File::open, read_at, tokio_uring::start の正式 API 確認
grep -rn "pub async fn open\|pub async fn read_at\|pub fn start" \
  ~/.cargo/registry/src/*/tokio-uring-0.4*/src/ 2>/dev/null | head -15
```

**tokio-uring 0.4 の主要 API（事前調査結果）:**

| API | 説明 |
|---|---|
| `tokio_uring::start(future)` | io_uring ランタイムを起動してブロック（同期呼び出し） |
| `tokio_uring::fs::File::open(path)` | 非同期ファイルオープン |
| `file.read_at(buf: Vec<u8>, offset: u64)` | 位置指定非同期読み込み。戻り値: `(io::Result<usize>, Vec<u8>)` |
| `std::fs::metadata(path)` | ファイルサイズ取得（同期、安価な syscall。`tokio_uring::fs::File::metadata()` は使わない） |

> API が異なる場合のフォールバック: `file.read(buf)` / `file.read_to_end(buf)` を調査する。

### 完了条件
- `cargo check` でコンパイルエラー 0（Linux 以外でも）
- Linux 環境で `use tokio_uring::fs::File;` がコンパイルを通る（CI または WSL2 で確認）

---

## T2: `vm.rs` — `read_files_batch_impl` + `IO.read_files_batch`

### 2-1. `FavList` 変換パターンの事前確認

```bash
grep -n "FavList::new\|FavList::from\|into_iter.*collect" \
  fav/src/backend/vm.rs | grep -v "//" | head -10
# → VMValue::List 構築に使われているパターンを確認
```

### 2-2. `read_files_batch_impl` を vm.rs に追加

配置場所: `// ── v20.5.0: mmap + arrow-csv helpers` セクションの直前または直後。

**Linux バックエンド:**

```rust
// ── v20.6.0: io_uring batch file reader ──────────────────────────────────────

/// Read multiple files concurrently.
/// On Linux: uses tokio-uring (io_uring syscall interface, near-zero context switches).
/// On other platforms: uses rayon parallel read_to_string (thread-pool fallback).
///
/// Returns files in the same order as the input paths.
/// Any single failure causes the whole batch to return Err.
#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
pub(crate) fn read_files_batch_impl(paths: &[String]) -> Result<Vec<String>, String> {
    tokio_uring::start(async {
        let futures: Vec<_> = paths.iter()
            .map(|p| read_one_uring(p.clone()))
            .collect();
        futures::future::try_join_all(futures).await
    })
    .map_err(|e| format!("IO.read_files_batch (io_uring): {e}"))
}

/// Read a single file using tokio-uring (Linux only).
#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
async fn read_one_uring(path: String) -> Result<String, String> {
    use tokio_uring::fs::File;

    // Get file size via std metadata (cheap syscall, not worth uring for a single stat)
    let size = std::fs::metadata(&path)
        .map_err(|e| format!("IO.read_files_batch: metadata '{}': {e}", path))?
        .len() as usize;

    let file = File::open(&path).await
        .map_err(|e| format!("IO.read_files_batch: open '{}': {e}", path))?;

    let buf = vec![0u8; size];
    let (res, mut buf) = file.read_at(buf, 0).await;
    let bytes_read = res.map_err(|e| format!("IO.read_files_batch: read '{}': {e}", path))?;
    buf.truncate(bytes_read);  // 部分読み込み対応（size < file_size の場合）

    String::from_utf8(buf)
        .map_err(|e| format!("IO.read_files_batch: utf8 '{}': {e}", path))
}

/// Non-Linux fallback: rayon parallel read (Windows / macOS).
#[cfg(all(not(target_os = "linux"), not(target_arch = "wasm32")))]
pub(crate) fn read_files_batch_impl(paths: &[String]) -> Result<Vec<String>, String> {
    use rayon::prelude::*;
    paths.par_iter()
        .map(|p| std::fs::read_to_string(p)
            .map_err(|e| format!("IO.read_files_batch: cannot read '{}': {e}", p)))
        .collect()
}
```

> **実装注意 — `read_at` のバッファサイズ**:
> `read_at(buf, 0)` は `buf.len()` バイトだけ読み込もうとする。
> ファイルサイズが 0 の場合は空文字列を返すように別途チェックが必要:
> ```rust
> if size == 0 {
>     return Ok(String::new());
> }
> ```
>
> **実装注意 — `futures` クレート**:
> `futures::future::try_join_all` は既存依存 `futures = "0.3"` があれば使用可能。
> なければ `tokio::task::JoinSet` または手動 join で代替。
> 事前確認: `grep "^futures" fav/Cargo.toml`

### 2-3. `"IO.read_files_batch"` を `vm_call_builtin` に追加

既存の `"IO.read_file_raw"` の近くに追加:

```rust
"IO.read_files_batch" => {
    let paths = match args.into_iter().next() {
        Some(VMValue::List(list)) => {
            list.iter()
                .map(|v| match v {
                    VMValue::Str(s) => Ok(s.clone()),
                    other => Err(format!(
                        "IO.read_files_batch: path must be String, got {}",
                        vmvalue_type_name(other)
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?
        }
        _ => return Err("IO.read_files_batch: expected List<String> argument".to_string()),
    };
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

> **`FavList` の収集方法**: `FavList` は `FromIterator` を実装していないため `collect::<FavList>()` は使えない。
> `FavList::new(contents.into_iter().map(VMValue::Str).collect::<Vec<_>>())` を使う（確定）。

### 2-4. tokio ランタイム競合の事前確認

```bash
grep -n "tokio::main\|block_on" fav/src/main.rs | head -10
```

`vm_call_builtin` が `#[tokio::main]` や `block_on` の内側から呼ばれていないことを確認する。
呼ばれていない場合（通常の VM ループから呼ばれる）、`tokio_uring::start()` は安全に使用できる。

### 2-5. `is_known_builtin_namespace` の確認

`"IO"` が既に `is_known_builtin_namespace` に登録されているか確認する:

```bash
grep -n "\"IO\"" fav/src/backend/vm.rs | head -5
```

登録済みであれば compiler.rs・checker.rs の変更は不要。

### 完了条件
- `cargo check` でコンパイルエラー 0
- Linux 以外（Windows）で `read_files_batch_impl` が rayon フォールバックを使う
- `"IO"` namespace が is_known_builtin_namespace 登録済み（変更不要）

---

## T3: `driver.rs` — `v206000_tests`

```rust
#[cfg(test)]
mod v206000_tests {
    use crate::backend::vm::read_files_batch_impl;

    // line!() でテストごとに一意なファイルパスを生成（並列実行時の競合回避）
    macro_rules! temp_txt {
        ($content:expr) => {{
            let mut p = std::env::temp_dir();
            p.push(format!("fav_v206_{}_{}.txt", std::process::id(), line!()));
            std::fs::write(&p, $content).unwrap();
            p
        }};
    }

    #[test]
    fn version_is_20_6_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("20.6.0"), "Cargo.toml should have version 20.6.0");
    }

    #[test]
    fn io_batch_reads_single_file() {
        let path = temp_txt!("hello world");
        let paths = vec![path.to_str().unwrap().to_string()];
        let result = read_files_batch_impl(&paths);
        let _ = std::fs::remove_file(&path);
        let contents = result.expect("should read single file");
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0], "hello world");
    }

    #[test]
    fn io_batch_reads_multiple_files() {
        let p1 = temp_txt!("file one");
        let p2 = temp_txt!("file two");
        let p3 = temp_txt!("file three");
        let paths = vec![
            p1.to_str().unwrap().to_string(),
            p2.to_str().unwrap().to_string(),
            p3.to_str().unwrap().to_string(),
        ];
        let result = read_files_batch_impl(&paths);
        let _ = std::fs::remove_file(&p1);
        let _ = std::fs::remove_file(&p2);
        let _ = std::fs::remove_file(&p3);
        let contents = result.expect("should read 3 files");
        assert_eq!(contents.len(), 3);
        // Order preservation is tested in io_batch_preserves_order
        assert!(contents.iter().any(|s| s == "file one"), "missing file one");
        assert!(contents.iter().any(|s| s == "file two"), "missing file two");
        assert!(contents.iter().any(|s| s == "file three"), "missing file three");
    }

    #[test]
    fn io_batch_preserves_order() {
        let p1 = temp_txt!("alpha");
        let p2 = temp_txt!("beta");
        let p3 = temp_txt!("gamma");
        let paths = vec![
            p1.to_str().unwrap().to_string(),
            p2.to_str().unwrap().to_string(),
            p3.to_str().unwrap().to_string(),
        ];
        let result = read_files_batch_impl(&paths);
        let _ = std::fs::remove_file(&p1);
        let _ = std::fs::remove_file(&p2);
        let _ = std::fs::remove_file(&p3);
        let contents = result.expect("should read in order");
        assert_eq!(contents[0], "alpha", "first file should be 'alpha'");
        assert_eq!(contents[1], "beta", "second file should be 'beta'");
        assert_eq!(contents[2], "gamma", "third file should be 'gamma'");
    }

    #[test]
    fn io_batch_error_on_missing_file() {
        let result = read_files_batch_impl(&[
            "/nonexistent/path/fav_v206_missing.txt".to_string(),
        ]);
        assert!(result.is_err(), "should return Err for missing file");
    }
}
```

### 完了条件
- `cargo test v206000` — 5/5 PASS

---

## T4: `fav/Cargo.toml` バージョン更新

`version = "20.5.0"` → `"20.6.0"` に変更。

既存の v20.5.0 バージョンテスト（`version_is_20_5_0`）に `#[ignore]` を追加。

---

## T5: `CHANGELOG.md` 更新 + `benchmarks/v20.6.0.json`

### CHANGELOG エントリ

```markdown
## [v20.6.0] — 2026-06-XX — io_uring 非同期 I/O（Linux）

### Added
- `IO.read_files_batch(paths: List<String>) -> List<String>` — 複数ファイル並列読み込み
  - Linux: `tokio-uring` (io_uring) によるゼロコンテキストスイッチ非同期 I/O
  - Windows / macOS: `rayon` 並列 `read_to_string` フォールバック
  - WASM: `err_vm` を返す（非対応）
- `read_files_batch_impl` ヘルパー関数（`pub(crate)`）— platform cfg で分岐
- `tokio-uring = "0.4"` 依存クレート追加（Linux 専用: `[target.'cfg(target_os = "linux")'.dependencies]`）

### Performance（Linux 本番環境）
- `io_batch_100_files_ms`: +2〜4x 改善（期待値）
- `io_batch_1000_files_ms`: +3〜5x 改善（期待値）
- `io_db_file_mixed_ms`: +1.5〜2x 改善（期待値）
- 実測は `benchmarks/v20.6.0.json` 参照
```

### ベンチマークスクリプトの新規作成

`benchmarks/suite/09_io_batch.sh` を新規作成し、以下を計測:
- `io_batch_100_files_ms` — 100 ファイル並列読み込み（ms）
- `io_batch_1000_files_ms` — 1000 ファイル並列読み込み（ms）
- `io_db_file_mixed_ms` — DB + ファイル混在ワークロード（ms）

### `benchmarks/v20.6.0.json`

実測後に生成:
```bash
bash benchmarks/suite/run_all.sh --format json > benchmarks/v20.6.0.json
```

Linux 本番環境で `io_batch_100_files_ms` が v20.5.0 比 +2x 以上であることを確認する。

---

## T6: `site/content/docs/runes/io.mdx` 更新

既存の `io.mdx` に `IO.read_files_batch` セクションを追加する。

```mdx
## IO.read_files_batch

```favnir
IO.read_files_batch(paths: List<String>) -> List<String>
```

複数のファイルを並列に読み込み、内容のリストを返します。
結果は入力パスと同じ順序で返されます。

```favnir
stage LoadShards: List<String> -> List<String> = |paths| {
  IO.read_files_batch(paths)
}
```

**プラットフォーム動作:**
- **Linux**: io_uring（カーネル 5.1+）による非同期並列 I/O
- **Windows / macOS**: rayon スレッドプールによる並列 `read_to_string`
- **WASM**: 非対応（`err` が返ります）

> **注意**: Linux 環境ではカーネル 5.1+ が必要です。カーネル 5.1 未満では実行時エラーになります。

**エラー処理**: いずれか 1 ファイルの読み込みに失敗した場合、全体が `err` になります。
```

---

## 注意点

### `tokio-uring` ランタイムの分離

`tokio-uring::start()` は既存の `tokio` ランタイムの外側でのみ使用可能。
既存の `#[tokio::main]` や `tokio::runtime::Handle::block_on` の内側では使えない。
`read_files_batch_impl` は VM スタックから呼ばれる同期関数のため、
`tokio_uring::start()` を直接使って問題ない（既存の tokio ランタイムに干渉しない）。

### `futures` クレートの依存確認

`futures::future::try_join_all` を使う場合:
```bash
grep "^futures" fav/Cargo.toml
```
存在しない場合は `futures = "0.3"` を native-only deps に追加するか、
以下の代替で実装する:

```rust
// futures なしで複数タスクを join する代替
let mut results = Vec::with_capacity(paths.len());
for p in paths {
    results.push(read_one_uring(p.clone()).await?);
}
results
```
（この場合は並列性が下がるが実装は単純になる。Phase 1 では許容範囲。）

### `io_batch_preserves_order` テストについて

非 Linux（Windows）では `rayon::par_iter` を使うが、rayon は順序を保証する
（`par_iter().map().collect()` は入力と同じ順序で結果を返す）ため、
順序テストは全プラットフォームで有効。

### Linux 環境での確認

開発環境が Windows の場合、Linux バックエンドは WSL2 または CI で確認する:
```bash
# WSL2 での確認
wsl cargo test v206000
```
CI では Linux ランナーで自動的に `tokio-uring` バックエンドが使われる。
