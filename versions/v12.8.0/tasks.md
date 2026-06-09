# Favnir v12.8.0 Tasks

Date: 2026-06-09
Theme: `fav scaffold <template>` — 正しい雛形生成

---

## Phase A — `scaffold_*` 関数群（driver.rs）

- [ ] A-1: `ScaffoldArgs` 構造体を定義（in_type / out_type / effect / out_file / stages）
- [ ] A-2: `scaffold_stage(name: &str, args: &ScaffoldArgs) -> String` を実装
  - デフォルト: `String -> String !IO`、`bind _result <- IO.println(input)` を含む
  - `--no-effect` 時はエフェクト注釈なし
  - コメント行にシグネチャを含める
- [ ] A-3: `scaffold_seq(name: &str, stages: &[&str]) -> String` を実装
  - デフォルト stages: `["Load", "Transform", "Save"]`
  - 最初と最後のステージに `!IO`、中間はエフェクトなし
  - `public seq Name = A |> B |> C` を末尾に出力
- [ ] A-4: `scaffold_postgres_etl() -> String` を実装
  - `chain _ <- IO.write_file_raw(...)` を使う（`bind _` 禁止）
  - LoadCsv / InsertRows / SaveResult / EtlPipeline を含む
- [ ] A-5: `scaffold_rune(name: &str) -> String` を実装
  - `public fn hello(name: String) -> String` を含む
- [ ] A-6: `write_scaffold(content: &str, out: Option<&str>)` を実装

---

## Phase B — `cmd_scaffold` ハンドラ（driver.rs）

- [ ] B-1: `cmd_scaffold(sub: &str, args: &[String])` を実装
  - `"stage"` / `"seq"` / `"postgres-etl"` / `"rune"` に分岐
  - 未知 sub → `eprintln!` + `exit(1)`
- [ ] B-2: 引数パーサを実装
  - `--in <Type>` / `--out-type <Type>` / `--effect <Effect>` / `--no-effect`
  - `--out <file>` / `--stages <A,B,C>`
  - 先頭の位置引数（Name）を取得

---

## Phase C — `fav new --template postgres-etl`（driver.rs）

- [ ] C-1: `POSTGRES_ETL_TOML` 定数を定義（`[project]` + `[postgres]` sslmode = "require"）
- [ ] C-2: `POSTGRES_ETL_MAIN_FAV` 定数を定義（`Main` stage + `EtlPipeline` 呼び出し）
- [ ] C-3: `try_cmd_new` の template match に `"postgres-etl"` 分岐を追加
  - `src/` ディレクトリ + `fav.toml` + `src/pipeline.fav` + `src/main.fav` を生成

---

## Phase D — main.rs の変更

- [ ] D-1: `Some("scaffold")` 分岐を追加
  - `args[2]` = sub コマンド、`args[3..]` を `cmd_scaffold` に渡す
  - sub が空の場合 usage を表示して exit 1
- [ ] D-2: `use crate::driver::cmd_scaffold;` をインポートに追加

---

## Phase E — テスト追加（driver.rs）

- [ ] E-1: `scaffold_stage_contains_public` — `"public stage"` が含まれること
- [ ] E-2: `scaffold_stage_has_effect` — デフォルトで `"!IO"` が含まれること
- [ ] E-3: `scaffold_stage_no_effect` — `--no-effect` 指定時エフェクトなし
- [ ] E-4: `scaffold_seq_has_pipe` — `"|>"` が含まれること
- [ ] E-5: `scaffold_seq_default_stages` — `"Load"` / `"Transform"` / `"Save"` が含まれること
- [ ] E-6: `scaffold_seq_custom_stages` — `stages = ["Fetch","Process","Push"]` で正しく生成
- [ ] E-7: `scaffold_postgres_etl_uses_chain` — `"chain"` が含まれること
- [ ] E-8: `scaffold_rune_contains_fn` — `"public fn"` が含まれること
- [ ] E-9: `new_template_postgres_etl_creates_dir` — tempdir に `fav.toml` と `src/pipeline.fav` が作成される
- [ ] E-10: `version_is_12_8_0` — `CARGO_PKG_VERSION == "12.8.0"`
- [ ] E-11: `cargo test v12800` — 10 件通過確認

---

## Phase F — バージョン更新・コミット

- [ ] F-1: `fav/Cargo.toml` version → `"12.8.0"`
- [ ] F-2: `cargo test` — 全通過
- [ ] F-3: `git commit -m "feat: v12.8.0 — fav scaffold <template>"`
- [ ] F-4: `git push` → CI 通過確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `fav scaffold stage MyStage` で `public stage` が出る | |
| `fav scaffold seq MyPipeline` で `\|>` が出る | |
| `fav scaffold postgres-etl` で `chain` ベースの ETL が出る | |
| `fav scaffold rune MyLib` で `public fn` が出る | |
| `fav new --template postgres-etl my-etl` でディレクトリが作成される | |
| `cargo test v12800` 10 件通過 | |
| `cargo test` 全通過 | |
