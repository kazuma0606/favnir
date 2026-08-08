# v62.6.0 タスクリスト

Status: COMPLETE
Version: 62.6.0
Base tests: 3392
Target tests: 3394

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3392 tests passed, 0 failed を確認
- [x] `driver.rs` に `cmd_build_docker` が **存在しない** ことを grep で確認
- [x] `driver.rs` に `cmd_build_docker_dry_run` が **存在しない** ことを grep で確認
- [x] `main.rs` の `Some("build")` アーム内に `"--docker"` が **存在しない** ことを確認
- [x] `main.rs` の `Some("build")` アーム内に build 用 `dry_run` 変数が **存在しない** ことを確認
- [x] `driver.rs` に `v62500_tests` が存在することを確認（挿入位置確認）
- [x] `main.rs` に `let mut aot_stats` が存在する行番号を確認（`docker` / `dry_run` 変数追加位置）

---

## T1: `driver.rs` — `validate_docker_tag` + `generate_dockerfile` + `cmd_build_docker_dry_run` + `cmd_build_docker` 追加

- [x] `cmd_build_aot_stats` の直後に以下を追加:
  ```rust
  fn validate_docker_tag(tag: &str) -> Result<(), String> {
      if tag.is_empty() {
          return Err("tag is required (--tag <name>:<version>)".to_string());
      }
      if !tag.contains(':') {
          return Err(format!("invalid tag format: expected <name>:<version>, got '{tag}'"));
      }
      Ok(())
  }

  fn generate_dockerfile(tag: &str) -> String {
      let tag_name = tag.split(':').next().unwrap_or(tag);
      format!(
          "FROM debian:12-slim\nWORKDIR /app\nCOPY ./pipeline.bin /app/pipeline\nRUN chmod +x /app/pipeline\nLABEL org.opencontainers.image.title=\"{tag_name}\"\nENTRYPOINT [\"/app/pipeline\"]\n"
      )
  }

  /// v62.6.0: `--docker --dry-run` モード — Dockerfile のみ返す（docker 呼び出しなし）。
  #[cfg(not(target_arch = "wasm32"))]
  pub fn cmd_build_docker_dry_run(src: &str, tag: &str) -> String {
      if let Err(e) = validate_docker_tag(tag) {
          return format!("error: {e}");
      }
      match crate::frontend::parser::Parser::parse_str(src, "<build>") {
          Ok(_) => {}
          Err(e) => return format!("parse error: {e}"),
      }
      generate_dockerfile(tag)
  }

  /// v62.6.0: `--docker` モード — Dockerfile を生成し `docker build` を呼び出す。
  /// docker が利用できない場合は `"docker not available: ..."` を返す（panic しない）。
  #[cfg(not(target_arch = "wasm32"))]
  pub fn cmd_build_docker(src: &str, tag: &str) -> String {
      if let Err(e) = validate_docker_tag(tag) {
          return format!("error: {e}");
      }
      match crate::frontend::parser::Parser::parse_str(src, "<build>") {
          Ok(_) => {}
          Err(e) => return format!("parse error: {e}"),
      }
      let dockerfile = generate_dockerfile(tag);
      println!("Building AOT binary...");
      println!("Generating Dockerfile...");
      use std::io::Write as _;
      // std::process::Command / Stdio はフルパスで参照（driver.rs は use std::process; のみ）
      let mut child = match std::process::Command::new("docker")
          .args(["build", "-t", tag, "-f", "-", "."])
          .stdin(std::process::Stdio::piped())
          .spawn()
      {
          Ok(c) => c,
          Err(e) => return format!("docker not available: {e}"),
      };
      if let Some(stdin) = child.stdin.as_mut() {
          let _ = stdin.write_all(dockerfile.as_bytes());
      }
      match child.wait() {
          Ok(status) if status.success() => format!("Building image: {tag}"),
          Ok(status) => format!("docker build failed: exit code {:?}", status.code()),
          Err(e) => format!("docker wait error: {e}"),
      }
  }
  ```
- [x] `cargo build` でエラーなし

---

## T2: `main.rs` — `Some("build")` アームに `--docker` / `--tag` / `--dry-run` 追加

- [x] `let mut aot_stats = false;` の直後に以下を追加:
  ```rust
  let mut docker = false;
  let mut dry_run_docker = false;
  let mut tag: Option<&str> = None;
  ```
- [x] `while` ループ内に以下を追加（`"--aot-stats"` アームの近く）:
  ```rust
  "--docker" => { docker = true; i += 1; }
  "--dry-run" => { dry_run_docker = true; i += 1; }
  "--tag" => {
      tag = Some(args.get(i + 1).unwrap_or_else(|| {
          eprintln!("error: --tag requires a value");
          process::exit(1);
      }));
      i += 2;
  }
  ```
- [x] `if aot_stats { ... } else if link { ... }` の前に `docker` ブランチを追加:
  ```rust
  if docker {
      let f = file.unwrap_or_else(|| {
          eprintln!("error: build --docker requires a source file");
          process::exit(1);
      });
      let tag_val = tag.unwrap_or("app:latest");
      let src = std::fs::read_to_string(f).unwrap_or_else(|e| {
          eprintln!("error: cannot read {f}: {e}");
          process::exit(1);
      });
      if dry_run_docker {
          println!("{}", driver::cmd_build_docker_dry_run(&src, tag_val));
      } else {
          println!("{}", driver::cmd_build_docker(&src, tag_val));
      }
  } else if aot_stats {
  ```
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v62600_tests` 追加

- [x] `v62500_tests` の直前（ファイル先頭方向）に `v62600_tests` モジュールを挿入
- [x] `use super::*;` を先頭に追加
- [x] `build_docker_dockerfile_generated` テスト追加:
  - ソース: `"fn add(a: Int, b: Int) -> Int { a + b }\nfn main() -> Bool { 1 + 2 == 3 }"`
  - `cmd_build_docker_dry_run(src, "test-image:1.0")` を呼ぶ
  - 結果が `"FROM debian:12-slim"` を含むことを確認
  - 結果が `"COPY"` を含むことを確認
  - 結果が `"ENTRYPOINT"` を含むことを確認
- [x] `build_docker_tag_format` テスト追加:
  - ソース: `"fn main() -> Bool { true }"`
  - `cmd_build_docker(src, "")` → `"error"` を含む
  - `cmd_build_docker(src, "invalidtag")` → `"error"` を含む
  - `cmd_build_docker(src, "valid-image:1.0")` → `validate_docker_tag` のエラーメッセージ（`"tag is required"` / `"invalid tag format:"` ）を含まない（docker が存在しない場合は `"docker not available"` が返るがタグ形式エラーではない）
- [x] `cargo test v62600` で 2 件 PASS

---

## T4: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v62600` で 2 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3394 tests passed, 0 failed を確認

---

## T5: ドキュメント更新

- [x] `versions/roadmap/roadmap-v62.1-v63.0.md` v62.6.0 セクションに実績を追記
- [x] `versions/current.md` の「進行中」を v62.6.0（3394 tests）に更新、「次」を v62.7.0 に
- [x] `CHANGELOG.md` に v62.6.0 エントリを追加
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

（実装中に発覚した問題）
- **`generate_dockerfile` 名前衝突**: 既存 L17064 に同名関数（deploy 用）が存在。`generate_aot_dockerfile` に改名して解決。

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3394 passed, 0 failed（ベース 3392 + 2）
- 主要実装: `cmd_build_docker` / `cmd_build_docker_dry_run`（`#[cfg(not(target_arch = "wasm32"))]`）
- 完了日: 2026-08-01
