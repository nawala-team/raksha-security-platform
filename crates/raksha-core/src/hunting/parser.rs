//! RQL Parser - Parses token stream into an Abstract Syntax Tree.
//!
//! Grammar (simplified):
//!   query       := source [where_clause] [group_clause] [time_clause] [order_clause] [limit_clause]
//!   source      := 'events' | 'alerts' | 'agents' | 'network'
//!   where_clause:= 'where' condition
//!   condition   := comparison (('and'|'or') comparison)*
//!   comparison  := 'not' comparison | identifier operator value | identifier 'in' '(' values ')'
//!                | identifier 'between' value 'and' value | identifier 'contains' string
//!                | identifier 'matches' string

use super::models::*;

/// Parser for RQL token streams.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    source: Option<QuerySource>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            source: None,
        }
    }

    /// Parse a complete RQL query string into an AST.
    pub fn parse_query(input: &str) -> Result<QueryAst, QueryValidationError> {
        let mut lexer = super::lexer::Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Self::new(tokens);
        parser.parse()
    }

    /// Parse tokens into a QueryAst.
    pub fn parse(&mut self) -> Result<QueryAst, QueryValidationError> {
        let source = self.parse_source()?;
        self.source = Some(source.clone());

        let filter = if self.check(&Token::Where) {
            self.advance();
            Some(WhereClause {
                condition: self.parse_condition()?,
            })
        } else {
            None
        };

        let aggregations = self.parse_aggregations()?;
        let time_range = self.parse_time_range()?;
        let order_by = self.parse_order_by()?;
        let limit = self.parse_limit()?;

        Ok(QueryAst {
            source,
            filter,
            aggregations,
            time_range,
            order_by,
            limit,
        })
    }

    fn parse_source(&mut self) -> Result<QuerySource, QueryValidationError> {
        match self.current() {
            Token::Events => {
                self.advance();
                Ok(QuerySource::Events)
            }
            Token::Alerts => {
                self.advance();
                Ok(QuerySource::Alerts)
            }
            Token::Agents => {
                self.advance();
                Ok(QuerySource::Agents)
            }
            Token::Network => {
                self.advance();
                Ok(QuerySource::Network)
            }
            _ => Err(self.error(
                "Expected data source: events, alerts, agents, or network",
                ValidationErrorKind::MissingSource,
            )),
        }
    }


    fn parse_condition(&mut self) -> Result<Condition, QueryValidationError> {
        self.parse_or_condition()
    }

    fn parse_or_condition(&mut self) -> Result<Condition, QueryValidationError> {
        let mut left = self.parse_and_condition()?;
        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and_condition()?;
            left = Condition::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and_condition(&mut self) -> Result<Condition, QueryValidationError> {
        let mut left = self.parse_unary_condition()?;
        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_unary_condition()?;
            left = Condition::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary_condition(&mut self) -> Result<Condition, QueryValidationError> {
        if self.check(&Token::Not) {
            self.advance();
            let cond = self.parse_unary_condition()?;
            return Ok(Condition::Not(Box::new(cond)));
        }
        if self.check(&Token::LeftParen) {
            self.advance();
            let cond = self.parse_condition()?;
            self.expect(&Token::RightParen)?;
            return Ok(cond);
        }
        self.parse_primary_condition()
    }

    fn parse_primary_condition(&mut self) -> Result<Condition, QueryValidationError> {
        let field = self.expect_identifier()?;

        if let Some(ref source) = self.source {
            if !SourceSchema::is_valid_field(source, &field) {
                return Err(self.error(
                    &format!("Unknown field '{}' for {:?}", field, source),
                    ValidationErrorKind::UnknownField,
                ));
            }
        }

        match self.current() {
            Token::In => {
                self.advance();
                self.expect(&Token::LeftParen)?;
                let values = self.parse_value_list()?;
                self.expect(&Token::RightParen)?;
                Ok(Condition::InList(InListExpr {
                    field, values, negated: false,
                }))
            }
            Token::Between => {
                self.advance();
                let low = self.parse_literal()?;
                self.expect(&Token::And)?;
                let high = self.parse_literal()?;
                Ok(Condition::Between(BetweenExpr { field, low, high }))
            }
            Token::Contains => {
                self.advance();
                let value = self.expect_string()?;
                Ok(Condition::Contains(ContainsExpr { field, value }))
            }
            Token::Matches => {
                self.advance();
                let pattern = self.expect_string()?;
                Ok(Condition::Matches(MatchesExpr { field, pattern }))
            }
            _ => {
                let operator = self.parse_comparison_op()?;
                let value = self.parse_literal()?;
                Ok(Condition::Comparison(ComparisonExpr {
                    field, operator, value,
                }))
            }
        }
    }

    fn parse_comparison_op(&mut self) -> Result<ComparisonOp, QueryValidationError> {
        let op = match self.current() {
            Token::Eq => ComparisonOp::Eq,
            Token::Neq => ComparisonOp::Neq,
            Token::Gt => ComparisonOp::Gt,
            Token::Lt => ComparisonOp::Lt,
            Token::Gte => ComparisonOp::Gte,
            Token::Lte => ComparisonOp::Lte,
            _ => {
                return Err(self.error(
                    "Expected comparison operator",
                    ValidationErrorKind::InvalidOperator,
                ));
            }
        };
        self.advance();
        Ok(op)
    }

    fn parse_literal(&mut self) -> Result<LiteralValue, QueryValidationError> {
        let value = match self.current() {
            Token::StringLiteral(s) => LiteralValue::String(s.clone()),
            Token::IntegerLiteral(n) => LiteralValue::Integer(*n),
            Token::NumberLiteral(n) => LiteralValue::Float(*n),
            Token::Duration(d) => LiteralValue::Duration(d.clone()),
            Token::Cidr(c) => LiteralValue::Cidr(c.clone()),
            _ => {
                return Err(self.error(
                    "Expected literal value",
                    ValidationErrorKind::InvalidLiteral,
                ));
            }
        };
        self.advance();
        Ok(value)
    }

    fn parse_value_list(&mut self) -> Result<Vec<LiteralValue>, QueryValidationError> {
        let mut values = vec![self.parse_literal()?];
        while self.check(&Token::Comma) {
            self.advance();
            values.push(self.parse_literal()?);
        }
        Ok(values)
    }

    fn parse_aggregations(&mut self) -> Result<Vec<Aggregation>, QueryValidationError> {
        let mut aggregations = Vec::new();

        if self.check(&Token::GroupBy) {
            self.advance();
            let mut group_fields = vec![self.expect_identifier()?];
            while self.check(&Token::Comma) {
                self.advance();
                group_fields.push(self.expect_identifier()?);
            }
            let (function, having) = if self.check(&Token::Count) {
                self.advance();
                let having = if self.is_comparison_op() {
                    let op = self.parse_comparison_op()?;
                    let value = self.parse_literal()?;
                    Some(HavingClause { operator: op, value })
                } else {
                    None
                };
                (AggregateFunction::Count, having)
            } else {
                (AggregateFunction::Count, None)
            };
            aggregations.push(Aggregation {
                function,
                field: None,
                group_by: group_fields,
                having,
            });
        } else if self.check(&Token::Count) {
            self.advance();
            let having = if self.is_comparison_op() {
                let op = self.parse_comparison_op()?;
                let value = self.parse_literal()?;
                Some(HavingClause { operator: op, value })
            } else {
                None
            };
            aggregations.push(Aggregation {
                function: AggregateFunction::Count,
                field: None,
                group_by: vec![],
                having,
            });
        }

        Ok(aggregations)
    }

    fn parse_time_range(&mut self) -> Result<Option<TimeRangeExpr>, QueryValidationError> {
        if !self.check(&Token::TimeRange) {
            return Ok(None);
        }
        self.advance();
        if self.check(&Token::Last) {
            self.advance();
        }
        let duration = match self.current() {
            Token::Duration(d) => d.clone(),
            _ => {
                return Err(self.error(
                    "Expected duration (e.g., 24h, 7d, 30m)",
                    ValidationErrorKind::InvalidDuration,
                ));
            }
        };
        self.advance();
        Ok(Some(TimeRangeExpr { duration }))
    }

    fn parse_order_by(&mut self) -> Result<Option<OrderByExpr>, QueryValidationError> {
        if !self.check(&Token::OrderBy) {
            return Ok(None);
        }
        self.advance();
        let field = self.expect_identifier()?;
        let direction = if self.check(&Token::Desc) {
            self.advance();
            SortDirection::Desc
        } else if self.check(&Token::Asc) {
            self.advance();
            SortDirection::Asc
        } else {
            SortDirection::Asc
        };
        Ok(Some(OrderByExpr { field, direction }))
    }

    fn parse_limit(&mut self) -> Result<Option<u64>, QueryValidationError> {
        if !self.check(&Token::Limit) {
            return Ok(None);
        }
        self.advance();
        match self.current() {
            Token::IntegerLiteral(n) => {
                let limit = *n as u64;
                self.advance();
                Ok(Some(limit))
            }
            _ => Err(self.error(
                "Expected integer after 'limit'",
                ValidationErrorKind::InvalidLiteral,
            )),
        }
    }


    // ============================================================
    // Helper methods
    // ============================================================

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn check(&self, expected: &Token) -> bool {
        std::mem::discriminant(self.current()) == std::mem::discriminant(expected)
    }

    fn is_comparison_op(&self) -> bool {
        matches!(
            self.current(),
            Token::Eq | Token::Neq | Token::Gt | Token::Lt | Token::Gte | Token::Lte
        )
    }

    fn expect(&mut self, expected: &Token) -> Result<(), QueryValidationError> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(
                &format!("Expected {:?}, found {:?}", expected, self.current()),
                ValidationErrorKind::UnexpectedToken,
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, QueryValidationError> {
        match self.current().clone() {
            Token::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            other => Err(self.error(
                &format!("Expected identifier, found {:?}", other),
                ValidationErrorKind::UnexpectedToken,
            )),
        }
    }

    fn expect_string(&mut self) -> Result<String, QueryValidationError> {
        match self.current().clone() {
            Token::StringLiteral(s) => {
                self.advance();
                Ok(s)
            }
            other => Err(self.error(
                &format!("Expected string literal, found {:?}", other),
                ValidationErrorKind::UnexpectedToken,
            )),
        }
    }

    fn error(&self, message: &str, kind: ValidationErrorKind) -> QueryValidationError {
        QueryValidationError {
            message: message.to_string(),
            position: self.pos,
            line: 1,
            column: self.pos + 1,
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let ast = Parser::parse_query(
            "events where severity = 'critical' time_range last 24h",
        ).unwrap();
        assert_eq!(ast.source, QuerySource::Events);
        assert!(ast.filter.is_some());
        assert!(ast.time_range.is_some());
        let tr = ast.time_range.unwrap();
        assert_eq!(tr.duration.amount, 24);
        assert_eq!(tr.duration.unit, DurationUnit::Hours);
    }

    #[test]
    fn test_parse_and_condition() {
        let ast = Parser::parse_query(
            "events where severity = 'critical' and source_ip in ('10.0.0.0/8') time_range last 24h",
        ).unwrap();
        assert_eq!(ast.source, QuerySource::Events);
        let filter = ast.filter.unwrap();
        assert!(matches!(filter.condition, Condition::And(_, _)));
    }

    #[test]
    fn test_parse_aggregation() {
        let ast = Parser::parse_query(
            "alerts where status = 'open' group_by agent_id count > 5",
        ).unwrap();
        assert_eq!(ast.source, QuerySource::Alerts);
        assert_eq!(ast.aggregations.len(), 1);
        let agg = &ast.aggregations[0];
        assert_eq!(agg.group_by, vec!["agent_id".to_string()]);
        assert!(agg.having.is_some());
    }

    #[test]
    fn test_parse_order_limit() {
        let ast = Parser::parse_query(
            "network where dst_port = 443 and bytes_out > 1000000 time_range last 1h order_by bytes_out desc limit 50",
        ).unwrap();
        assert_eq!(ast.source, QuerySource::Network);
        assert_eq!(ast.limit, Some(50));
        let order = ast.order_by.unwrap();
        assert_eq!(order.field, "bytes_out");
        assert_eq!(order.direction, SortDirection::Desc);
    }

    #[test]
    fn test_unknown_field_error() {
        let result = Parser::parse_query("events where fake_field = 'test'");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, ValidationErrorKind::UnknownField);
    }
}
