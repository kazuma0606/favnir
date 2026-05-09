# Runes

Favnir の標準ルーンライブラリ。

**設計方針**: ルーンは Rust の VM builtins に依存せず、Favnir 自身で書く。
VM 側には最小限のプリミティブ（`Random.int` / `Random.float` 等）のみ追加し、
その上の高レベルロジックはすべて `.fav` ファイルで実装する。

## ステータス

| ルーン | 状態 | 依存 |
|---|---|---|
| `validate` | 未実装（Favnir で実装予定） | なし（純粋ロジック） |
| `stat` | 未実装（Favnir で実装予定） | `Random` VM primitive のみ |

## 利用方法（v2.6.0 モジュールシステム実装後）

```favnir
import rune "validate"
import rune "stat"
```

## ディレクトリ構成

```
runes/
  validate/    -- バリデーションルーン
  stat/        -- 統計・型駆動生成ルーン
```
