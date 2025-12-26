// Does not include all fields in
// https://www.elastic.co/docs/api/doc/elasticsearch/operation/operation-search#operation-search-responses
#[derive(Debug, serde::Deserialize)]
pub struct OperationSearchResult {
    #[serde(rename = "took")]
    pub time_took_ms: serde_json::Number, 
    pub timed_out: bool,
    #[serde(rename = "_shards")]
    pub shards_used: OperationSearchShardsUsed,
    pub hits: OperationSearchHits,
    #[serde(default)]
    pub aggregations: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
pub struct OperationSearchShardsUsed {
    pub failed: serde_json::Number,
    pub successful: serde_json::Number,
    pub total: serde_json::Number,
}

#[derive(Debug, serde::Deserialize)]
pub struct OperationSearchHits {
    pub hits: Vec<serde_json::Value>,
}
