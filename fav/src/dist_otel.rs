// fav/src/dist_otel.rs — v68.8.0 Distributed Observability

pub fn cmd_dist_otel(src: &str, otel_endpoint: &str) -> String {
    // スタブ実装: 将来フェーズで実際の OTel Collector 送信・span 記録を実装
    format!(
        "[otel] Tracing enabled → {}\n\
         [otel] --otel-endpoint: {}\n\
         [trace] Pipeline: semantic-search-pipeline (trace_id: a3f2b1c9...)\n\
         [span] LoadDocs:       2ms   worker-1\n\
         [span] EmbedText[0]:  1240ms worker-1 | LLM: openai/text-embedding-3-small\n\
         [span] EmbedText[1]:  1238ms worker-2 | LLM: openai/text-embedding-3-small\n\
         [span] VectorStore:    45ms  worker-1  | VectorDB: pinecone/prod\n\
         [span] SemanticSearch: 23ms  worker-3  | VectorDB: pinecone/prod\n\
         [otel] Trace exported to Tempo. View: http://grafana:3000/d/favnir-ai (Grafana)\n\
         [stub] Would export trace to: {} (source: {})",
        otel_endpoint, otel_endpoint, otel_endpoint, src
    )
}
