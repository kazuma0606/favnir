# v62.6.0 Plan — Docker / OCI イメージ生成（`fav build --docker`）

Version: 62.6.0
Status: 未着手

---

## 実装順序

### Step 1: `driver.rs` — `validate_docker_tag` + `generate_dockerfile` + `cmd_build_docker_dry_run` + `cmd_build_docker` 追加

`cmd_build_aot_stats` の直後（または `mean_ms` / `p99_ms` の直前）に配置。
private helper を先に定義し、pub fn がそれらを呼ぶ構造にする。
`cargo build` でエラーなし確認。

### Step 2: `main.rs` — `Some("build")` アームに `--docker` / `--dry-run` 分岐追加

`let mut aot_stats = false;` の近くに `let mut docker = false; let mut dry_run = false;` を追加。
ループ内に `"--docker"` / `"--dry-run"` アームを追加。
`if aot_stats` ブランチの直前に `if docker` ブランチを挿入。
`cargo build` でエラーなし確認。

### Step 3: `driver.rs` — `v62600_tests` 追加

`v62500_tests` の直前（ファイル先頭方向）に挿入。
`cargo test v62600` で 2 件 PASS 確認。

### Step 4: 全テスト

`cargo test -j 8 -- --test-threads=8` で 3394 tests passed, 0 failed を確認。

### Step 5: ドキュメント更新

roadmap / current.md / CHANGELOG.md / tasks.md を更新。

---

## 設計メモ

### `--tag` の CLI パース

既存の `target: Option<&str>` 変数を `--target` フラグで設定しているが、
`--docker` 向けには `--tag <name>:<ver>` として使いたい。
`--tag` を新しいフラグとして追加するか、`--target` をそのまま流用するか。

**決定**: `--tag` を独立した変数 `tag: Option<&str>` として追加する。
`target` は既存の graphql/proto/schema 用途と分離することで混乱を防ぐ。

```rust
let mut tag: Option<&str> = None;
// ループ内:
"--tag" => { tag = Some(args.get(i + 1).unwrap_or_else(|| { ... })); i += 2; }
```

docker ブランチ:
```rust
let tag = tag.unwrap_or("app:latest");
```

### `cmd_build_docker` の docker 呼び出し

テスト環境（Windows / CI）では docker が利用できない場合が多い。
`std::process::Command::new("docker")` の失敗は `Err(e)` で捕捉して
`"docker not available: {e}"` を返す。テストは `cmd_build_docker_dry_run` または
タグバリデーションエラーを使ってdocker 呼び出しを回避する。

### `build_docker_tag_format` テスト — 有効タグの確認方法

`cmd_build_docker(src, "valid-image:1.0")` はdocker を呼び出すが、
結果がタグ形式エラー（`"error: invalid tag"` 等）を**含まない**ことを確認する。
docker が存在しない場合は `"docker not available"` が返るが、
これはタグ形式エラーではないためテストをパスする。

### ロードマップとの乖離

- `cmd_build_docker_dry_run` はロードマップで関数名が明記されていないが、`--dry-run` モードの実装関数として追加（ロードマップの意図の範囲内）。
- AOT binary の実際の埋め込みは非スコープ（`COPY ./pipeline.bin` はプレースホルダー）。
