# versions/v30-v35/

v30.1.0 〜 v35.0.0 のバージョン別ドキュメントを収録するディレクトリ。

各バージョン実装時に以下の構造で作成します:

```
versions/v30-v35/
├── v30.1.0/
│   ├── spec.md      バージョン仕様書
│   ├── plan.md      実装計画書
│   └── tasks.md     タスクリスト（実装完了後 COMPLETE に更新）
├── v30.2.0/
│   └── ...
...
└── v35.0.0/
    └── ...
```

テンプレートは `versions/_templates/version/` を参照。

マスタースケジュール: `versions/roadmap/roadmap-v30.1-v35.0.md`
