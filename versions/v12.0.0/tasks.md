# Favnir v12.0.0 Tasks

Date: 2026-06-06
Theme: Python トランスパイラ完成宣言 + fav2py E2E PASS

---

## Phase A — fav2py E2E 最終確認

- [ ] A-1: `infra/e2e-demo/fav2py/scripts/verify.sh` — PASS=5 以上を確認
  - [1] Fav ネイティブ版の S3 出力が存在する
  - [2] Python トランスパイル版の S3 出力が存在する
  - [3] 両者の集計結果（JSON）が一致する
  - [4] RDS への INSERT 件数が一致する
  - [5] ECS タスクが正常終了している
- [ ] A-2: `infra/e2e-demo/fav2py/tasks.md` — Phase 8 完了確認

---

## Phase B — CHANGELOG.md 更新

- [ ] B-1: `CHANGELOG.md` 先頭に `[v12.0.0]` エントリ追加
- [ ] B-2: `[v11.9.0]` 〜 `[v11.1.0]` の全エントリを追記

---

## Phase C — README.md 更新

- [ ] C-1: 機能一覧に `fav transpile --target python` を追加
- [ ] C-2: ロードマップ表に `v11.1.0〜v11.9.0` / `v12.0.0` 行を追記
- [ ] C-3: エフェクト → Python ライブラリ対応表を追加

---

## Phase D — ドキュメント

- [ ] D-1: `site/content/docs/transpile/python.mdx` 新規作成
  - `fav transpile --target python` 使用方法
  - uv との組み合わせ（仮想環境 → 検証 → デプロイ）
  - エフェクト → Python ライブラリ対応表
  - fav2py E2E デモへのリンク
- [ ] D-2: `site/content/docs/effects/postgres.mdx` 新規作成
  - `!Postgres` エフェクトリファレンス
  - `fav.toml [postgres]` 設定
  - `fav infer --from postgres` 使用例

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| fav2py E2E — PASS=5 以上 | |
| `fav transpile --target python` ドキュメント公開 | |
| CHANGELOG / README 更新済み | |
