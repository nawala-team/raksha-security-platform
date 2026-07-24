//! RQL Executor - Converts AST to OpenSearch DSL and executes queries.
//!
//! Translates parsed RQL queries into OpenSearch JSON DSL, sends them
//! via HTTP, and returns typed results with pagination support.

use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use super::models::*;
use super::parser::Parser;

/// Configuration for the OpenSearch connection.
#[derive(Debug, Clone)]
pub struct OpenSearchConfig {
    pub base_url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub timeout_ms: u64,
    pub max_result_window: u64,
}

impl Default for OpenSearchConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:9200".to_string(),
            username: None,
            password: None,
            timeout_ms: 30_000,
            max_result_window: 10_000,
        }
    }
}

/// Query executor that compiles RQL to OpenSearch DSL.
pub struct QueryExecutor {
    config: OpenSearchConfig,
    http_client: reqwest::Client,
}

impl QueryExecutor {
    pub fn new(config: OpenSearchConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .expect("Failed to build HTTP client");

        Self { config, http_client }
    }

    /// Parse, compile, and execute an RQL query.
    pub async fn execute(
        &self,
        query_text: &str,
        offset: u64,
        limit: u64,
    ) -> Result<HuntResult, QueryValidationError> {
        let ast = Parser::parse_query(query_text)?;
        let dsl = self.compile_to_dsl(&ast, offset, limit);
        let index = ast.source.index_name();

        let start = std::time::Instant::now();
        let response = self.send_query(index, &dsl).await?;
        let execution_time_ms = start.elapsed().as_millis() as u64;

        self.parse_response(response, offset, limit, execution_time_ms)
    }

    /// Compile an AST into OpenSearch DSL JSON (useful for debugging).
    pub fn compile(&self, query_text: &str) -> Result<Value, QueryValidationError> {
        let ast = Parser::parse_query(query_text)?;
        Ok(self.compile_to_dsl(&ast, 0, 100))
    }

    /// Build the OpenSearch DSL query from the AST.
    fn compile_to_dsl(&self, ast: &QueryAst, offset: u64, limit: u64) -> Value {
        let mut query = json!({});
        let mut must_clauses: Vec<Value> = Vec::new();

        // Build filter conditions
        if let Some(ref filter) = ast.filter {
            let filter_dsl = self.condition_to_dsl(&filter.condition);
            must_clauses.push(filter_dsl);
        }

        // Build time range filter
        if let Some(ref time_range) = ast.time_range {
            let range_filter = json!({
                "range": {
                    "@timestamp": {
                        "gte": time_range.duration.to_opensearch_date_math(),
                        "lte": "now"
                    }
                }
            });
            must_clauses.push(range_filter);
        }

        // Assemble bool query
        if must_clauses.is_empty() {
            query["query"] = json!({"match_all": {}});
        } else if must_clauses.len() == 1 {
            query["query"] = json!({"bool": {"must": must_clauses}});
        } else {
            query["query"] = json!({"bool": {"must": must_clauses}});
        }

        // Pagination
        let effective_limit = ast.limit.unwrap_or(limit).min(self.config.max_result_window);
        query["from"] = json!(offset);
        query["size"] = json!(effective_limit);

        // Sort order
        if let Some(ref order) = ast.order_by {
            let dir = match order.direction {
                SortDirection::Asc => "asc",
                SortDirection::Desc => "desc",
            };
            query["sort"] = json!([{&order.field: {"order": dir}}]);
        } else {
            query["sort"] = json!([{"@timestamp": {"order": "desc"}}]);
        }

        // Aggregations
        if !ast.aggregations.is_empty() {
            query["aggs"] = self.aggregations_to_dsl(&ast.aggregations);
        }

        query
    }

    /// Convert a condition tree to OpenSearch DSL.
    fn condition_to_dsl(&self, condition: &Condition) -> Value {
        match condition {
            Condition::Comparison(expr) => self.comparison_to_dsl(expr),
            Condition::InList(expr) => {
                let values: Vec<Value> = expr.values.iter()
                    .map(|v| self.literal_to_value(v))
                    .collect();
                let clause = json!({"terms": {&expr.field: values}});
                if expr.negated {
                    json!({"bool": {"must_not": [clause]}})
                } else {
                    clause
                }
            }
            Condition::Between(expr) => {
                json!({
                    "range": {
                        &expr.field: {
                            "gte": self.literal_to_value(&expr.low),
                            "lte": self.literal_to_value(&expr.high)
                        }
                    }
                })
            }
            Condition::Contains(expr) => {
                json!({"wildcard": {&expr.field: format!("*{}*", expr.value)}})
            }
            Condition::Matches(expr) => {
                json!({"regexp": {&expr.field: &expr.pattern}})
            }
            Condition::And(left, right) => {
                json!({
                    "bool": {
                        "must": [
                            self.condition_to_dsl(left),
                            self.condition_to_dsl(right)
                        ]
                    }
                })
            }
            Condition::Or(left, right) => {
                json!({
                    "bool": {
                        "should": [
                            self.condition_to_dsl(left),
                            self.condition_to_dsl(right)
                        ],
                        "minimum_should_match": 1
                    }
                })
            }
            Condition::Not(inner) => {
                json!({"bool": {"must_not": [self.condition_to_dsl(inner)]}})
            }
        }
    }

    /// Convert a comparison expression to OpenSearch DSL.
    fn comparison_to_dsl(&self, expr: &ComparisonExpr) -> Value {
        let value = self.literal_to_value(&expr.value);
        match expr.operator {
            ComparisonOp::Eq => json!({"term": {&expr.field: value}}),
            ComparisonOp::Neq => {
                json!({"bool": {"must_not": [{"term": {&expr.field: value}}]}})
            }
            ComparisonOp::Gt => {
                json!({"range": {&expr.field: {"gt": value}}})
            }
            ComparisonOp::Lt => {
                json!({"range": {&expr.field: {"lt": value}}})
            }
            ComparisonOp::Gte => {
                json!({"range": {&expr.field: {"gte": value}}})
            }
            ComparisonOp::Lte => {
                json!({"range": {&expr.field: {"lte": value}}})
            }
        }
    }

    /// Convert a literal to a serde_json Value.
    fn literal_to_value(&self, lit: &LiteralValue) -> Value {
        match lit {
            LiteralValue::String(s) => json!(s),
            LiteralValue::Integer(n) => json!(n),
            LiteralValue::Float(f) => json!(f),
            LiteralValue::Bool(b) => json!(b),
            LiteralValue::Cidr(c) => json!(c),
            LiteralValue::Duration(d) => json!(d.to_seconds()),
        }
    }

    /// Build aggregation DSL from aggregation list.
    fn aggregations_to_dsl(&self, aggregations: &[Aggregation]) -> Value {
        let mut aggs = json!({});
        for (i, agg) in aggregations.iter().enumerate() {
            let agg_name = format!("agg_{}", i);
            if !agg.group_by.is_empty() {
                let group_field = &agg.group_by[0];
                let mut terms_agg = json!({
                    "terms": {"field": group_field, "size": 100}
                });
                // Nested count with having
                if agg.having.is_some() {
                    terms_agg["aggs"] = json!({
                        "count_filter": {"bucket_selector": {
                            "buckets_path": {"count": "_count"},
                            "script": self.having_to_script(agg.having.as_ref().unwrap())
                        }}
                    });
                }
                aggs[agg_name] = terms_agg;
            } else {
                aggs[agg_name] = json!({"value_count": {"field": "_id"}});
            }
        }
        aggs
    }

    fn having_to_script(&self, having: &HavingClause) -> String {
        let op = match having.operator {
            ComparisonOp::Gt => ">",
            ComparisonOp::Lt => "<",
            ComparisonOp::Gte => ">=",
            ComparisonOp::Lte => "<=",
            ComparisonOp::Eq => "==",
            ComparisonOp::Neq => "!=",
        };
        let value = match &having.value {
            LiteralValue::Integer(n) => n.to_string(),
            LiteralValue::Float(f) => f.to_string(),
            _ => "0".to_string(),
        };
        format!("params.count {} {}", op, value)
    }


    /// Send the compiled DSL query to OpenSearch.
    async fn send_query(
        &self,
        index: &str,
        dsl: &Value,
    ) -> Result<Value, QueryValidationError> {
        let url = format!("{}/{}/_search", self.config.base_url, index);

        let mut request = self.http_client.post(&url)
            .header("Content-Type", "application/json")
            .json(dsl);

        if let (Some(ref user), Some(ref pass)) = (&self.config.username, &self.config.password) {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await.map_err(|e| {
            QueryValidationError {
                message: format!("OpenSearch request failed: {}", e),
                position: 0,
                line: 0,
                column: 0,
                kind: ValidationErrorKind::SyntaxError,
            }
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(QueryValidationError {
                message: format!("OpenSearch returned {}: {}", status, body),
                position: 0,
                line: 0,
                column: 0,
                kind: ValidationErrorKind::SyntaxError,
            });
        }

        response.json::<Value>().await.map_err(|e| {
            QueryValidationError {
                message: format!("Failed to parse OpenSearch response: {}", e),
                position: 0,
                line: 0,
                column: 0,
                kind: ValidationErrorKind::SyntaxError,
            }
        })
    }

    /// Parse the OpenSearch response into a HuntResult.
    fn parse_response(
        &self,
        response: Value,
        offset: u64,
        limit: u64,
        execution_time_ms: u64,
    ) -> Result<HuntResult, QueryValidationError> {
        let total = response["hits"]["total"]["value"]
            .as_u64()
            .unwrap_or(0);

        let hits: Vec<Value> = response["hits"]["hits"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|hit| {
                        let mut doc = hit["_source"].clone();
                        if let Some(obj) = doc.as_object_mut() {
                            obj.insert(
                                "_id".to_string(),
                                hit["_id"].clone(),
                            );
                            obj.insert(
                                "_index".to_string(),
                                hit["_index"].clone(),
                            );
                        }
                        doc
                    })
                    .collect()
            })
            .unwrap_or_default();

        let aggregations = response.get("aggregations").cloned();

        Ok(HuntResult {
            query_id: Uuid::now_v7(),
            execution_id: Uuid::now_v7(),
            executed_at: Utc::now(),
            execution_time_ms,
            total_hits: total,
            hits,
            aggregations,
            pagination: ResultPagination {
                offset,
                limit,
                total,
                has_more: offset + limit < total,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_query() {
        let executor = QueryExecutor::new(OpenSearchConfig::default());
        let dsl = executor.compile(
            "events where severity = 'critical' time_range last 24h",
        ).unwrap();

        assert!(dsl["query"]["bool"]["must"].is_array());
        assert!(dsl["sort"].is_array());
        assert_eq!(dsl["from"], 0);
        assert_eq!(dsl["size"], 100);
    }

    #[test]
    fn test_compile_network_query() {
        let executor = QueryExecutor::new(OpenSearchConfig::default());
        let dsl = executor.compile(
            "network where dst_port = 443 and bytes_out > 1000000 time_range last 1h order_by bytes_out desc limit 50",
        ).unwrap();

        assert_eq!(dsl["size"], 50);
        // Sort should be by bytes_out desc
        let sort = &dsl["sort"][0];
        assert!(sort["bytes_out"].is_object());
    }

    #[test]
    fn test_compile_aggregation_query() {
        let executor = QueryExecutor::new(OpenSearchConfig::default());
        let dsl = executor.compile(
            "alerts where status = 'open' group_by agent_id count > 5",
        ).unwrap();

        assert!(dsl["aggs"].is_object());
        assert!(dsl["aggs"]["agg_0"]["terms"].is_object());
    }

    #[test]
    fn test_compile_in_list_query() {
        let executor = QueryExecutor::new(OpenSearchConfig::default());
        let dsl = executor.compile(
            "events where source_ip in ('10.0.0.0/8') time_range last 24h",
        ).unwrap();

        // Should produce a terms query
        let must = &dsl["query"]["bool"]["must"];
        assert!(must.is_array());
    }
}
