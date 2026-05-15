- [x] 0
- [x] 1
- [x] 2
- [x] 3
- [x] 4
- [x] 5
- [x] 6
- [x] 7
- [x] 8

完了:
- Phase 0: バージョン更新（3.3.0）
- Phase 1: DbError 型 + DbHandle/TxHandle 不透明型 + effect Db
- Phase 2: SQLite VM プリミティブ（DB.connect / query_raw / execute_raw / begin_tx 等 11 関数）
- Phase 3: PostgreSQL VM プリミティブ（postgres クレート追加 optional feature + E0605 stub）
- Phase 4: Env.get / Env.get_or VM プリミティブ
- Phase 5: runes/db/db.fav + db.test.fav（8 テスト）
- Phase 6: checker 型チェック統合 + L008 リンタ（E0601〜E0605）
- Phase 7: examples/db_demo + driver.rs 統合テスト（7 テスト）
- Phase 8: langspec.md / migration-guide.md 作成
