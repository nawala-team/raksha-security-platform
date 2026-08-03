//! Domain models for the RQL threat hunting engine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// Token & AST Types
// ============================================================

/// Token types produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Events,
    Alerts,
    Agents,
    Network,
    Where,
    And,
    Or,
    Not,
    In,
    Between,
    Contains,
    Matches,
    Count,
    GroupBy,
    TimeRange,
    OrderBy,
    Limit,
    Last,
    Asc,
    Desc,

    // Operators
    Eq,       // =
    Neq,      // !=
    Gt,       // >
    Lt,       // <
    Gte,      // >=
    Lte,      // <=

    // Literals
    StringLiteral(String),
    NumberLiteral(f64),
    IntegerLiteral(i64),
    Duration(DurationValue),
    Cidr(String),

    // Identifiers
    Identifier(String),

    // Punctuation
    LeftParen,
    RightParen,
    Comma,

    // End of input
    Eof,
}

/// Duration value with unit.
#[derive(Debug, Clone, PartialEq)]
pub struct DurationValue {
    pub amount: u64,
    pub unit: DurationUnit,
}

impl DurationValue {
    pub fn to_seconds(&self) -> u64 {
        match self.unit {
            DurationUnit::Seconds => self.amount,
            DurationUnit::Minutes => self.amount * 60,
            DurationUnit::Hours => self.amount * 3600,
            DurationUnit::Days => self.amount * 86400,
        }
    }

    pub fn to_opensearch_date_math(&self) -> String {
        let suffix = match self.unit {
            DurationUnit::Seconds => "s",
            DurationUnit::Minutes => "m",
            DurationUnit::Hours => "h",
            DurationUnit::Days => "d",
        };
        format!("now-{}{}", self.amount, suffix)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DurationUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

// ============================================================
// AST Nodes
// ============================================================

/// Root AST node for a parsed RQL query.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryAst {
    pub source: QuerySource,
    pub filter: Option<WhereClause>,
    pub aggregations: Vec<Aggregation>,
    pub time_range: Option<TimeRangeExpr>,
    pub order_by: Option<OrderByExpr>,
    pub limit: Option<u64>,
}

/// Data source to query against.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuerySource {
    Events,
    Alerts,
    Agents,
    Network,
}

impl QuerySource {
    pub fn index_name(&self) -> &'static str {
        match self {
            QuerySource::Events => "raksha-events-*",
            QuerySource::Alerts => "raksha-alerts-*",
            QuerySource::Agents => "raksha-agents-*",
            QuerySource::Network => "raksha-network-*",
        }
    }
}

/// WHERE clause containing condition tree.
#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    pub condition: Condition,
}

/// Condition tree with logical combinators.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Comparison(ComparisonExpr),
    InList(InListExpr),
    Between(BetweenExpr),
    Contains(ContainsExpr),
    Matches(MatchesExpr),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}

/// Field comparison expression (field op value).
#[derive(Debug, Clone, PartialEq)]
pub struct ComparisonExpr {
    pub field: String,
    pub operator: ComparisonOp,
    pub value: LiteralValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOp {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
}

/// IN expression (field in ('val1', 'val2')).
#[derive(Debug, Clone, PartialEq)]
pub struct InListExpr {
    pub field: String,
    pub values: Vec<LiteralValue>,
    pub negated: bool,
}

/// BETWEEN expression (field between val1 and val2).
#[derive(Debug, Clone, PartialEq)]
pub struct BetweenExpr {
    pub field: String,
    pub low: LiteralValue,
    pub high: LiteralValue,
}

/// CONTAINS expression (field contains 'substring').
#[derive(Debug, Clone, PartialEq)]
pub struct ContainsExpr {
    pub field: String,
    pub value: String,
}

/// MATCHES expression (field matches 'regex_pattern').
#[derive(Debug, Clone, PartialEq)]
pub struct MatchesExpr {
    pub field: String,
    pub pattern: String,
}

/// Literal values in expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Integer(i64),
    Float(f64),
    Duration(DurationValue),
    Cidr(String),
    Bool(bool),
}

/// Aggregation expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Aggregation {
    pub function: AggregateFunction,
    pub field: Option<String>,
    pub group_by: Vec<String>,
    pub having: Option<HavingClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

/// HAVING clause for aggregate filtering.
#[derive(Debug, Clone, PartialEq)]
pub struct HavingClause {
    pub operator: ComparisonOp,
    pub value: LiteralValue,
}

/// Time range expression.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeRangeExpr {
    pub duration: DurationValue,
}

/// ORDER BY expression.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByExpr {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

// ============================================================
// Domain Models
// ============================================================

/// A complete hunt query with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntQuery {
    pub id: Uuid,
    pub query_text: String,
    pub source: QuerySource,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub execution_time_ms: Option<u64>,
}

/// A saved/scheduled query for recurring threat hunts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuery {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub query_text: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub schedule: Option<QuerySchedule>,
    pub alert_on_results: bool,
    pub tags: Vec<String>,
    pub enabled: bool,
}

/// Cron-like schedule for query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySchedule {
    /// Cron expression (e.g., "0 */5 * * *" for every 5 minutes)
    pub cron_expression: String,
    /// Timezone for schedule evaluation
    pub timezone: String,
    /// Last successful execution
    pub last_run_at: Option<DateTime<Utc>>,
    /// Next planned execution
    pub next_run_at: Option<DateTime<Utc>>,
}

/// Result of a hunt query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntResult {
    pub query_id: Uuid,
    pub execution_id: Uuid,
    pub executed_at: DateTime<Utc>,
    pub execution_time_ms: u64,
    pub total_hits: u64,
    pub hits: Vec<serde_json::Value>,
    pub aggregations: Option<serde_json::Value>,
    pub pagination: ResultPagination,
}

/// Pagination metadata for results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultPagination {
    pub offset: u64,
    pub limit: u64,
    pub total: u64,
    pub has_more: bool,
}

/// Query validation errors with source location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryValidationError {
    pub message: String,
    pub position: usize,
    pub line: usize,
    pub column: usize,
    pub kind: ValidationErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationErrorKind {
    UnexpectedToken,
    UnknownField,
    InvalidOperator,
    InvalidLiteral,
    InvalidDuration,
    InvalidCidr,
    UnclosedString,
    MissingSource,
    MissingWhereClause,
    InvalidAggregation,
    SyntaxError,
}

impl std::fmt::Display for QueryValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RQL Error at {}:{}: {} ({:?})",
            self.line, self.column, self.message, self.kind
        )
    }
}

impl std::error::Error for QueryValidationError {}

// ============================================================
// Schema Validation
// ============================================================

/// Known fields for each data source, used for query validation.
pub struct SourceSchema;

impl SourceSchema {
    pub fn fields_for(source: &QuerySource) -> &'static [&'static str] {
        match source {
            QuerySource::Events => &[
                "id", "timestamp", "severity", "source_ip", "dest_ip",
                "source_port", "dest_port", "event_type", "action",
                "agent_id", "host", "user", "process", "file_path",
                "file_hash", "registry_key", "command_line",
                "parent_process", "dns_query", "http_method", "http_url",
                "http_status", "bytes_in", "bytes_out", "protocol",
                "rule_id", "mitre_tactic", "mitre_technique",
            ],
            QuerySource::Alerts => &[
                "id", "timestamp", "severity", "status", "title",
                "description", "agent_id", "rule_id", "source_ip",
                "dest_ip", "mitre_tactic", "mitre_technique",
                "assigned_to", "acknowledged_at", "resolved_at",
                "false_positive", "score",
            ],
            QuerySource::Agents => &[
                "id", "hostname", "ip_address", "os", "os_version",
                "agent_version", "status", "last_seen", "enrolled_at",
                "group", "tags", "cpu_usage", "memory_usage", "disk_usage",
            ],
            QuerySource::Network => &[
                "id", "timestamp", "src_ip", "dst_ip", "src_port",
                "dst_port", "protocol", "bytes_in", "bytes_out",
                "packets_in", "packets_out", "duration_ms", "action",
                "direction", "geo_src", "geo_dst", "dns_query",
                "tls_version", "tls_cipher", "http_host", "application",
            ],
        }
    }

    /// Check if a field is valid for the given source.
    pub fn is_valid_field(source: &QuerySource, field: &str) -> bool {
        Self::fields_for(source).contains(&field)
    }
}

