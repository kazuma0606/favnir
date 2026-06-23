// Favnir Official Rune Packages — static registry page (v24.7.0)

const rune_catalog = [
  { name: "auth",            version: "1.0.0", description: "JWT / OAuth2 / API Key 認証" },
  { name: "aws",             version: "1.0.0", description: "AWS SDK 統合（S3 / SQS / DynamoDB / Lambda）" },
  { name: "avro",            version: "1.0.0", description: "Apache Avro シリアライゼーション" },
  { name: "azure",           version: "1.0.0", description: "Azure SDK 統合（Blob / Queue / Functions）" },
  { name: "azure-servicebus",version: "1.0.0", description: "Azure Service Bus メッセージング" },
  { name: "bigquery",        version: "1.0.0", description: "Google BigQuery 統合" },
  { name: "bytes",           version: "1.0.0", description: "バイナリデータ操作" },
  { name: "cache",           version: "1.0.0", description: "インメモリ / Redis キャッシュ" },
  { name: "csv",             version: "1.0.0", description: "CSV 読み書き（ストリーミング対応）" },
  { name: "duckdb",          version: "1.0.0", description: "DuckDB 組み込み分析データベース" },
  { name: "dynamodb",        version: "1.0.0", description: "Amazon DynamoDB NoSQL" },
  { name: "email",           version: "1.0.0", description: "SMTP / SES メール送信" },
  { name: "excel",           version: "1.0.0", description: "Excel / XLSX 読み書き" },
  { name: "fs",              version: "1.0.0", description: "ファイルシステム操作" },
  { name: "gcs",             version: "1.0.0", description: "Google Cloud Storage" },
  { name: "grpc",            version: "1.0.0", description: "gRPC クライアント / サーバー" },
  { name: "http",            version: "1.0.0", description: "HTTP クライアント / サーバー" },
  { name: "huggingface",     version: "1.0.0", description: "HuggingFace API（推論エンドポイント）" },
  { name: "io",              version: "1.0.0", description: "標準入出力・ファイル IO" },
  { name: "json",            version: "1.0.0", description: "JSON エンコード / デコード" },
  { name: "kafka",           version: "1.0.0", description: "Apache Kafka プロデューサー / コンシューマー" },
  { name: "llm",             version: "1.0.0", description: "LLM 統合（Claude / OpenAI）" },
  { name: "mongodb",         version: "1.0.0", description: "MongoDB ドライバー" },
  { name: "mut",             version: "1.0.0", description: "可変状態（Mut セル）" },
  { name: "mysql",           version: "1.0.0", description: "MySQL / MariaDB ドライバー" },
  { name: "opentelemetry",   version: "1.0.0", description: "OpenTelemetry トレーシング / メトリクス" },
  { name: "orc",             version: "1.0.0", description: "Apache ORC カラム型フォーマット" },
  { name: "parquet",         version: "1.0.0", description: "Apache Parquet 読み書き" },
  { name: "postgres",        version: "1.0.0", description: "PostgreSQL ドライバー（TLS 対応）" },
  { name: "pubsub",          version: "1.0.0", description: "Google Cloud Pub/Sub" },
  { name: "queue",           version: "1.0.0", description: "インメモリ / SQS キュー" },
  { name: "redis",           version: "1.0.0", description: "Redis クライアント" },
  { name: "s3",              version: "1.0.0", description: "Amazon S3 オブジェクトストレージ" },
  { name: "scikit",          version: "1.0.0", description: "scikit-learn モデル推論" },
  { name: "slack",           version: "1.0.0", description: "Slack API（メッセージ送信）" },
  { name: "snowflake",       version: "1.0.0", description: "Snowflake データウェアハウス（JWT 認証）" },
  { name: "sql",             version: "1.0.0", description: "汎用 SQL Rune（型付きクエリ）" },
  { name: "sqlite",          version: "1.0.0", description: "SQLite 組み込みデータベース" },
  { name: "sqs",             version: "1.0.0", description: "Amazon SQS メッセージキュー" },
  { name: "state",           version: "1.0.0", description: "アプリケーション状態管理" },
  { name: "toml",            version: "1.0.0", description: "TOML 設定ファイル解析" },
  { name: "uuid",            version: "1.0.0", description: "UUID v4 / v7 生成" },
  { name: "websocket",       version: "1.0.0", description: "WebSocket クライアント / サーバー" },
  { name: "xml",             version: "1.0.0", description: "XML パーサー / ジェネレーター" },
  { name: "yaml",            version: "1.0.0", description: "YAML 設定ファイル解析" },
] as const;

export default function PackagesPage() {
  return (
    <main className="mx-auto max-w-5xl px-4 py-8">
      <h1 className="text-3xl font-bold mb-2">Favnir Rune Packages</h1>
      <p className="text-gray-600 mb-6">
        Official rune packages for Favnir — install with{" "}
        <code className="bg-gray-100 px-1 rounded">fav install &lt;name&gt;</code>
      </p>
      <p className="text-sm text-gray-500 mb-8">
        {rune_catalog.length} packages available
      </p>
      <table className="w-full text-sm border-collapse">
        <thead>
          <tr className="border-b text-left">
            <th className="py-2 pr-4 font-semibold">Name</th>
            <th className="py-2 pr-4 font-semibold">Version</th>
            <th className="py-2 font-semibold">Description</th>
          </tr>
        </thead>
        <tbody>
          {rune_catalog.map((pkg) => (
            <tr key={pkg.name} className="border-b hover:bg-gray-50">
              <td className="py-2 pr-4 font-mono text-blue-600">{pkg.name}</td>
              <td className="py-2 pr-4 text-gray-500">{pkg.version}</td>
              <td className="py-2">{pkg.description}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </main>
  );
}
