- [x] 0
- [x] 1
- [x] 2
- [x] 3
- [x] 4
- [x] 5
- [x] 6
- [x] 7

完了:
- Phase 0: バージョン更新（3.4.0）
- Phase 1: CLI プラミング（`fav infer` サブコマンド）
- Phase 2: 型推論コア（InferredType / format_type_def / table_name_to_type_name）
- Phase 3: CSV 推論（infer_from_csv、csv クレート使用）
- Phase 4: SQLite スキーマ推論（PRAGMA table_info、nullable → Option<T>）
- Phase 5: PostgreSQL スキーマ推論（information_schema、postgres_integration feature ゲート）
- Phase 6: `--out` ファイル/ディレクトリ出力 + エラー整備
- Phase 7: examples/infer_demo + langspec.md / migration-guide.md 作成
