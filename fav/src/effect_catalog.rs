/// SAP Platform Era エフェクトマーカーカタログ（v95.3.0〜）
/// pipeline シグネチャのエフェクトマーカー（!SapEvent 等）を文字列定数として定義する。
/// NOTE: Rust の Effect enum は v35.4.0 で削除済み（body call 推論に移行）のため、
///       本ファイルは定数カタログとして管理する。

/// SAP Event Mesh へのアクセスを伴う pipeline に付与するエフェクトマーカー
pub const SAP_EVENT: &str = "SapEvent";

/// SAP Analytics Cloud へのデータプッシュを伴う pipeline に付与するエフェクトマーカー
pub const SAP_ANALYTICS: &str = "SapAnalytics";
