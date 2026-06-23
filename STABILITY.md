# Favnir Stability Policy

## v1.x 後方互換性保証

v1.x（マイナーバージョン）では破壊的変更を行わない。
パッチバージョン（v1.x.y+1）はバグ修正のみ。

具体的な保証:
- 既存の `.fav` ソースファイルはそのままコンパイルできる
- 既存の Rune インターフェースは変更しない
- `fav run` / `fav check` / `fav lint` の CLI フラグを削除しない

## v2.0 破壊的変更ポリシー

破壊的変更は **2 年前から** `#[deprecated]` アノテーションで事前警告する。
`#[deprecated]` 付き API は `fav lint` で W020 警告が表示される。
削除予定は CHANGELOG.md と DEPRECATIONS.md に記録する。

例:
```favnir
#[deprecated]
fn old_api(x: Int) -> String { f"{x}" }
```

上記の関数を呼び出すと `fav lint` が W020 を報告する:
```
W020: call to deprecated function `old_api`
```

## SemVer 準拠

Favnir は Semantic Versioning 2.0.0 に完全準拠する。

```
MAJOR.MINOR.PATCH
  │     │     └── バグ修正のみ（後方互換）
  │     └──────── 機能追加（後方互換）
  └────────────── 破壊的変更（2 年前に deprecation 警告）
```

## `--legacy` フラグ

`--legacy` フラグは v2.0 まで維持する（v1.x では削除しない）。
v1.x ユーザーは `--legacy` で旧挙動にアクセスできる。

## 問い合わせ

互換性に関する質問・問題は GitHub Issues へ:
`https://github.com/favnir/fav/issues`
