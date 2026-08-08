# v62.6.0 Spec — Docker / OCI イメージ生成（`fav build --docker`）

Version: 62.6.0
Status: 未着手
Base tests: 3392
Target tests: 3394

---

## 概要

`fav build <file> --docker --tag <name>:<ver>` で Dockerfile を自動生成し `docker build` を呼び出す。
`fav build <file> --docker --dry-run` で Dockerfile のみ標準出力に表示するモードを追加。
ベースイメージは `debian:12-slim`、AOT binary のみを含む最小構成とする。

---

## 前提確認（T0 で実施）

- `driver.rs` に `cmd_build_docker` が **存在しない** ことを確認
- `driver.rs` に `cmd_build_docker_dry_run` が **存在しない** ことを確認
- `main.rs` の `Some("build")` アーム内に `--docker` が **存在しない** ことを確認
- `main.rs` の `Some("build")` アーム内に `--dry-run`（build 用）が **存在しない** ことを確認
- `driver.rs` に `v62500_tests` が存在することを確認（挿入位置確認）
- `cargo test -j 8 -- --test-threads=8` でベース 3392 tests passed, 0 failed を確認

---

## 実装スコープ

### 1. `driver.rs` — ヘルパー + `cmd_build_docker` + `cmd_build_docker_dry_run` 追加

**`validate_docker_tag(tag: &str) -> Result<(), String>`**（private fn）:
- `tag.is_empty()` → `Err("tag is required")`
- `!tag.contains(':')` → `Err("invalid tag format: expected <name>:<version>")`
- それ以外 → `Ok(())`

**`generate_dockerfile(tag: &str) -> String`**（private fn）:
```
FROM debian:12-slim
WORKDIR /app
COPY ./pipeline.bin /app/pipeline
RUN chmod +x /app/pipeline
LABEL org.opencontainers.image.title="{tag_name}"
ENTRYPOINT ["/app/pipeline"]
```
`tag_name` は `tag.split(':').next().unwrap_or(tag)` で取得。

**`cmd_build_docker_dry_run(src: &str, tag: &str) -> String`**（pub fn、`#[cfg(not(target_arch = "wasm32"))]` 付与）:
1. `validate_docker_tag(tag)` → Err なら `"error: {e}"` で早期リターン
2. parse src → Err なら `"parse error: {e}"` で早期リターン
3. `generate_dockerfile(tag)` を返す

**`cmd_build_docker(src: &str, tag: &str) -> String`**（pub fn、`#[cfg(not(target_arch = "wasm32"))]` 付与）:
1. `validate_docker_tag(tag)` → Err なら `"error: {e}"` で早期リターン
2. parse src → Err なら `"parse error: {e}"` で早期リターン
3. `generate_dockerfile(tag)` を生成
4. `println!("Building AOT binary...")` / `println!("Generating Dockerfile...")`
5. `std::process::Command::new("docker").args(["build", "-t", tag, "-f", "-", "."]).stdin(Stdio::piped()).status()` を試みる
   - `Ok(status)` if success → `format!("Building image: {tag}")`
   - `Ok(status)` if failed → `format!("docker build failed: exit code {:?}", status.code())`
   - `Err(e)` (docker not found 等) → `format!("docker not available: {e}")`

**注意**: `docker build` は `--dry-run` モードでない限り実際に呼ばれるが、CLI 環境に docker がない場合は `Err` で graceful にエラーを返す。テストでは `cmd_build_docker_dry_run` または tag フォーマットエラーを使って docker 呼び出しを回避する。

### 2. `main.rs` — `Some("build")` アームに `--docker` / `--dry-run` 分岐追加

変数宣言部に追加（`dry_run_docker` は `main.rs` 他アームの `dry_run` 変数と名前衝突を避けるため独自名）:
```rust
let mut docker = false;
let mut dry_run_docker = false;
let mut tag: Option<&str> = None;
```

`while` ループ内に追加:
```rust
"--docker" => { docker = true; i += 1; }
"--dry-run" => { dry_run = true; i += 1; }
```

`if aot_stats { ... } else if link { ... }` の前に追加:
```rust
} else if docker {
    let f = file.unwrap_or_else(|| { eprintln!("error: build --docker requires a source file"); process::exit(1); });
    let tag = target.unwrap_or("app:latest");
    let src = std::fs::read_to_string(f).unwrap_or_else(|e| { eprintln!("error: cannot read {f}: {e}"); process::exit(1); });
    if dry_run {
        println!("{}", driver::cmd_build_docker_dry_run(&src, tag));
    } else {
        println!("{}", driver::cmd_build_docker(&src, tag));
    }
```

`--tag` は `let mut tag: Option<&str> = None;` として独立変数を追加する（`--target` は graphql/proto/schema 用途と分離）。

### 3. `driver.rs` — `v62600_tests` 追加

`v62500_tests` の直前（ファイル先頭方向）に挿入。

**`build_docker_dockerfile_generated`**:
- ソース: `"fn add(a: Int, b: Int) -> Int { a + b }\nfn main() -> Bool { 1 + 2 == 3 }"`
- `cmd_build_docker_dry_run(src, "test-image:1.0")` を呼ぶ
- 結果が `"FROM debian:12-slim"` を含むことを確認
- 結果が `"COPY"` を含むことを確認
- 結果が `"ENTRYPOINT"` を含むことを確認

**`build_docker_tag_format`**:
- ソース: `"fn main() -> Bool { true }"`
- 空タグ: `cmd_build_docker(src, "")` → `"error"` を含む
- コロンなしタグ: `cmd_build_docker(src, "invalidtag")` → `"error"` を含む
- 有効タグ: `cmd_build_docker(src, "valid-image:1.0")` → `"error: tag"` や `"invalid tag"` を**含まない**（docker 呼び出しは失敗可だがタグ形式エラーではない）

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v62600` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3394 tests passed, 0 failed

---

## 非スコープ

- 実際の AOT binary を Dockerfile に埋め込む（`COPY ./pipeline.bin` はプレースホルダー）
- multi-stage Dockerfile（builder stage での AOT コンパイル）
- OCI manifest / イメージ署名
- `--platform` クロスプラットフォームビルド
- `site/content/docs/` の MDX ドキュメント — v62.9.0 でまとめて作成
