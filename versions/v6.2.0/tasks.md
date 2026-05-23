# Favnir v6.2.0 タスクリスト — 真のブートストラップ

作成日: 2026-05-22

## 概要

v6.1.0 で Favnir 製コンパイラ（compiler.fav）が hello.fav を非ゼロのバイト列にコンパイルできることを確認した。
v6.2.0 では完全な 3 ステージ ブートストラップを完成させる。

### ブートストラップの定義

```
Stage 1 : Rust VM が compiler.fav（ソース）を実行 → hello.fav をコンパイル → bytecode_A
Stage 2 : Rust VM が compiler.fav（ソース）を実行 → compiler.fav 自身をコンパイル → compiler_artifact
Stage 3 : Rust VM が compiler_artifact を実行 → hello.fav をコンパイル → bytecode_B

完了条件: bytecode_A == bytecode_B
```

### 現状の課題

1. **出力フォーマット不足**: compiler.fav は現在 `List<Int>` の生バイト列のみ出力。
   Rust VM が実行するには関数テーブル・文字列テーブルを含む `FvcArtifact` 形式が必要。
2. **自己コンパイル未確認**: compiler.fav が compiler.fav 自身を入力として処理できるか不明。
3. **Stage 3 実行基盤なし**: 生成した artifact バイト列を Rust VM にロードして実行するパスが未実装。

---

## Phase A: 現状把握

- [x] A-1: `FvcArtifact` のシリアライズ形式を確認（`codegen.rs` / `vm.rs` の `.fvc` 読み書き実装）
- [x] A-2: compiler.fav が hello.fav に対して生成する 24 バイトの内容を解析
  - Rust codegen が生成するバイト列と比較し、一致しているか確認
  - **発見**: 呼び出し規約ミスマッチ → `CallNamed = 0x56` で解決
- [x] A-3: compiler.fav を入力として compiler.fav を実行（Stage 2 試行）
  - **発見**: スタックオーバーフロー（1600行のファイルで再帰パーサが限界）
- [x] A-4: 調査結果をまとめ、Phase B〜E の実装方針を確定する

---

## Phase B: アーティファクト形式の拡張（compiler.fav 側）

compiler.fav が出力する `List<Int>` を、Rust VM が直接ロードできる
シリアライズ済み FvcArtifact 形式に拡張する。

- [x] B-1: `FnEntry` 型を compiler.fav に追加（name: String, arity: Int, code: List<Int>）
- [x] B-2: `Artifact` 型を追加（fns: List<FnEntry>, str_table: List<String>）
- [x] B-3: `compile()` の返値を `List<Int>` → `Artifact` に変更
- [x] B-4: `serialize_artifact(a: Artifact) -> List<Int>` を実装（FvcArtifact 形式）
- [x] B-5: `main()` で `print_bytes` 経由でバイト列を stdout に出力（Rust テストが受け取る）
- [x] B-6: `fav check fav/self/compiler.fav` エラーなし確認 (995 tests passing)
- [x] B-7: `fav run compiler.fav -- hello.fav` が "compiled: 201" + 201 バイトを出力することを確認

---

## Phase C: Rust 側ローダー確認・追加

- [ ] C-1: `FvcArtifact::from_bytes(bytes: &[u8])` が存在するか確認
  - 存在しなければ実装（`to_bytes()` の逆変換）
- [ ] C-2: compiler.fav の serialize_artifact 出力と Rust の from_bytes が互換であることをユニットテストで確認
  - hello.fav を compiler.fav でコンパイル → バイト列 → from_bytes → Rust codegen の Artifact と比較

---

## Phase D: 自己コンパイル対応（Stage 2 の完成）

compiler.fav が compiler.fav 自身をコンパイルできるようにする。

- [ ] D-1: Stage 2 試行（A-3 の結果を受けて対応）
  - compiler.fav が使っている Favnir 構文のうち、compiler.fav の codegen が未対応のものを列挙
- [ ] D-2: 不足している codegen 機能を compiler.fav に追加（例: stage 呼び出し、再帰、大きなリスト等）
- [ ] D-3: `fav run compiler.fav -- compiler.fav compiler.fvc` が完走する
- [ ] D-4: `compiler.fvc` が正常なアーティファクトとして Rust VM にロードできることを確認

---

## Phase E: Stage 3 実行・一致検証

- [ ] E-1: Stage 3 の実行パスを Rust テストとして実装
  - `compiler.fvc` をロード → Rust VM で実行 → hello.fav をコンパイル → bytecode_B を取得
- [ ] E-2: bytecode_A（Stage 1）と bytecode_B（Stage 3）の一致を assert
- [ ] E-3: `cargo test bootstrap_full_self_hosting` が通ることを確認

---

## Phase F: まとめ

- [ ] F-1: `cargo test` 全件通過
- [ ] F-2: `versions/v6.2.0/tasks.md` にチェックを入れる
- [ ] F-3: `MEMORY.md` を更新
- [ ] F-4: `feat: full bootstrap verified — Favnir compiler bootstraps itself (v6.2.0)` でコミット
