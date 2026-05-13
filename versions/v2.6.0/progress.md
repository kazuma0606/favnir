- [x] 0
- [x] 1
- [x] 2
- [x] 3
- [x] 4
- [x] 5
- [x] 6
- [x] 7

完了内容:
- Phase 0: バージョン更新
- Phase 1: レキサーに `TokenKind::Import` 追加
- Phase 2: AST に `Item::ImportDecl` 追加
- Phase 3: パーサーに `parse_import_decl` 追加
- Phase 4: チェッカーに namespace 解決と `E080` / `E081` 追加
- Phase 5: `fav check --dir` を driver / main に追加
- Phase 6: lexer / parser / checker / driver テスト追加
- Phase 7: 最終確認と `langspec.md` 作成
