# E2E デモ分類

Date: 2026-06-07

## 目的

`infra/e2e-demo/` 配下のシナリオは、それぞれ証明したいことが異なります。  
そのため、全部を同じ意味で「Favnir のテスト」と扱うべきではありません。

この文書では、各シナリオを検証軸ごとに分類します。目的は次の 3 つです。

- Favnir 本体の正しさと、インフラ実験を分ける
- CI/CD の品質ゲートを重くしすぎない
- 重いシナリオも reference architecture や R&D lab として repo 内に残せるようにする

## 検証軸

### 1. Favnir 近接検証

主に Favnir そのものに近い内容を見ているシナリオです。

- artifact 実行モデル
- runtime image から `.fav` ソースを分離できているか
- 現実的な packaging / deployment 下での runtime 挙動
- 制約のある環境でも Favnir がコンパイル済み pipeline を正しく動かせるか

unit test ではありませんが、プロダクト検証には比較的近い位置です。

### 2. Platform / Integration 検証

Favnir を、より大きなシステムの一部として組み込んだ時に成立するかを見るシナリオです。

- ECS / EKS / Lambda の配置パターン
- secrets, IAM, VPC endpoint, storage handoff
- RDS, S3, Snowflake のような managed service との接続

reference 実装としては有用ですが、失敗原因が Favnir ではなくインフラ側にあることも多いです。

### 3. Architecture / R&D Lab

ソリューション設計そのものを検証する色が強いシナリオです。

- マルチクラウド構成
- identity federation
- 外部 SaaS 連携
- 署名付きリクエストや verifier 設計
- migration フローや運用設計

戦略上の価値は高いですが、プロダクト品質ゲートには最も向きません。

## 運用カテゴリ

### A. Core Quality Gate

将来的に、プロダクト変更を block してもよいレベルの品質ゲートです。

条件:

- 失敗した時に Favnir の回帰を疑いやすい
- 実行時間が比較的小さい
- 外部クラウド制御面への依存が少ない

### B. Reference Integration

repo に置いておく価値はあるが、通常開発を block するべきではないものです。

条件:

- 対応する deployment style を示せる
- 実ユーザー導入時の構成を診断する助けになる
- クラウド依存はある程度許容される

### C. R&D Lab

探索、設計検証、将来提案向けの実験として扱うものです。

条件:

- 重い、実験的、不安定でもよい
- プロダクトよりアーキテクチャを検証している比重が高い
- 非 gating であることを明示する

## 現在の分類

| シナリオ | 主な検証軸 | 運用カテゴリ | 理由 |
|---|---|---|---|
| `lambda/` | Favnir 近接 + Platform | Reference Integration | artifact / runtime 実行と event-driven packaging の検証として強い。ただし AWS 依存が重いため core gate にはしない。 |
| `ecs/` | Platform | Reference Integration | 長時間実行・task compute 上で Favnir をどう使うかの検証として有用。`lambda/` ほど直接的に core correctness を示すわけではない。 |
| `eks/` | Platform | Reference Integration | Kubernetes / Fargate 配置パターンの妥当性確認として有用。主眼は platform fit であり、core regression 検知には向かない。 |
| `airgap/` | Favnir 近接 + Platform | Reference Integration | runtime 最小化、ソース非同梱、閉域動作の証明として非常に強い。プロダクト近接の証拠として価値が高い。 |
| `fav2py/` | Favnir 近接 | Reference Integration | transpilation と runtime parity を示す検証として強い。プロダクト方向性の検証として有用だが、通常の gate には重い。 |
| `snowflake/` | Platform | Reference Integration | 外部 warehouse 連携の実証として有用。Favnir 本体より integration の比重が高い。 |
| `crosscloud/` | Architecture / R&D | R&D Lab | Cognito + Entra ID + HMAC + AWS->Azure migration は、主にアーキテクチャ検証であり Favnir correctness そのものではない。 |

## 何を「Favnir テスト」と呼ぶべきか

これは「何を証明したいか」で変わります。

### Favnir 検証として扱ってよいもの

次の問いに答えたいなら、Favnir 検証として十分意味があります。

- `.fav` ソースを同梱せずに Favnir artifact を実行できるか
- Lambda / ECS / EKS のような環境に Favnir を載せられるか
- 現実的なデータパイプラインを Favnir で動かせるか

この観点では、`lambda/`, `airgap/`, `fav2py/` の一部は明確に Favnir 検証です。

### Favnir の core correctness としては扱わない方がよいもの

次の問いに答えたいなら、`infra/e2e-demo/` の大半は向いていません。

- 言語 / runtime / compiler が回帰していないか
- 通常のコード変更を block すべきか

理由は単純で、Cloud IAM、network、IdP federation、provider の癖、service availability など、Favnir 以外の失敗面が大きすぎるためです。

## 実務上の扱い

### Favnir 近接として扱うもの

- `lambda/`
- `airgap/`
- `fav2py/` の一部フロー

artifact packaging、runtime deployment、Favnir 実行モデルを主張したい時に使うのが適切です。

### Reference Integration として扱うもの

- `ecs/`
- `eks/`
- `snowflake/`

Favnir が実際の platform pattern に乗ることを示す用途に向いています。

### R&D Lab として扱うもの

- `crosscloud/`
- 将来の Azure -> AWS TiDB 逆方向 migration
- 将来の M365 / SharePoint / SaaS 連携シナリオ

solution architecture、security boundary、対外提案向けパターンを検証する用途です。

## CI / ガバナンス上の推奨

### デフォルトの CI gate から外してよいもの

- 現状の `infra/e2e-demo/` 全体

### 定期実行や手動実行候補として有力なもの

- `lambda/`
- `airgap/`
- `fav2py/`

### 明示的に非 gating にしておくべきもの

- `ecs/`
- `eks/`
- `snowflake/`
- `crosscloud/`

## cross-cloud シナリオの扱い

`crosscloud/` は次のように説明するのが適切です。

- `reference architecture`
- `integration lab`
- `security / migration R&D`

これは Favnir の core テストと呼ぶべきではありません。  
価値は十分ありますが、その価値は「Favnir が secure な cross-cloud migration pattern の中核になれるか」を示す点にあり、parser / checker / runtime の正しさを直接証明する点にはありません。
