# ADR-004: 副作用をエフェクト型で追跡する

- **状態**: 採用
- **決定日**: v7.3.0（`!Queue` / `!Cache` エフェクト導入時）
- **更新**: v9.5.0（`!Http`）、v9.6.0（`!Llm`）、v10.x（`!Snowflake` 予定）

---

## 背景

Favnir はデータパイプライン専用言語として設計されており、
ファイル I/O・HTTP・DB・キューなどの副作用を多用する。
型システムで副作用を追跡しないと、意図せず副作用を起こす関数が書けてしまう。

## 決定

副作用を持つ関数はシグネチャにエフェクトを宣言する。
エフェクトなしで副作用を持つ関数を定義することを禁止する。

```favnir
// エフェクト宣言あり（許可）
fn fetch(url: String) -> Result<String, String> !Http { ... }

// エフェクト宣言なし（型エラー E0311）
fn fetch(url: String) -> Result<String, String> { Http.get_body_raw(url) }
```

## 現在のエフェクト一覧

| エフェクト | 対象 | 導入 |
|---|---|---|
| `!IO` | ファイル読み書き | v7.5.0 |
| `!Network` | HTTP・gRPC・GraphQL（旧称、後方互換のため残存） | v5.x |
| `!Http` | HTTP・gRPC・GraphQL（推奨） | v9.5.0 |
| `!Queue` | メッセージキュー（SQS 等） | v7.3.0 |
| `!Cache` | キャッシュ（ElastiCache 等） | v7.3.0 |
| `!Llm` | LLM API（Claude / OpenAI） | v9.6.0 |
| `!Snowflake` | Snowflake クエリ | v10.x 予定 |
| `!AWS` | AWS SDK 全般 | v10.x 予定 |

## 理由

1. **パイプラインの副作用を明示化**: `stage` の型シグネチャを見るだけで
   何の外部サービスに接触するかがわかる。

2. **テスト容易性**: エフェクト宣言のある関数は、テスト時にモック可能。

3. **リネージ解析との統合**: `fav explain --lineage` がエフェクトを利用して
   外部依存を静的に解析できる。

4. **意図しない副作用の防止**: 純粋関数として意図した関数が
   誤って副作用を持つことをコンパイル時に検出できる。

## 却下した選択肢

- **エフェクトなし（副作用を型で追跡しない）**: データパイプラインの文脈では
  副作用の可視化は必須。追跡しない設計はバグ発見が遅れる。
- **モナド方式**: Favnir のパイプライン構文と相性が悪く、
  データエンジニアには馴染みにくい。

## 新しいエフェクトを追加するときのルール

1. `fav/src/frontend/ast.rs` の `Effect` enum に追加
2. `fav/src/frontend/parser.rs` でパース
3. `fav/src/middle/checker.rs` で型検査
4. `fav/self/checker.fav` に対応する型シグネチャを追加
5. `fav/src/backend/vm.rs` に primitive を追加
6. `runes/` 以下に Favnir 層を実装
7. `INVARIANTS.md` のエフェクト一覧を更新

この ADR の「現在のエフェクト一覧」も更新する。

## 影響

- エフェクトのない副作用は E0311 エラーとして検出される
- `INVARIANTS.md` II-4 にエフェクト完全性として記載
