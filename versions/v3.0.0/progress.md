- [ ] 0
- [ ] 1
- [x] 2
- [x] 3
- [x] 4
- [x] 5
- [x] 6
- [x] 7
- [x] 8

未完了:
- Phase 0: バージョン更新
- Phase 1: エラーコード移行（E0xxx 体系）— 全ソースの 3 桁コードを 4 桁に置換
- Phase 2: fav explain-error コマンド（error_catalog.rs 新規作成）
- Phase 3: explain JSON スキーマ v3.0（schema_version, trfs→stages, flws→seqs）
- Phase 4: selfhost lexer 完成（全トークン対応）
- Phase 5: selfhost parser 基礎実装（ast.fav, parser.fav, parser.test.fav, main.fav 新規作成）
- Phase 6: driver.rs 統合テスト（selfhost lexer/parser の実行テスト）
- Phase 7: fav explain compiler コマンド（5 ステップのコンパイル工程サマリー）
- Phase 8: langspec.md, migration-guide.md 作成
