//! RQL Lexer - Tokenizes Raksha Query Language input.
//!
//! Handles keywords, operators, string/number/duration/CIDR literals,
//! identifiers, and punctuation with precise position tracking.

use super::models::{
    DurationUnit, DurationValue, QueryValidationError, Token, ValidationErrorKind,
};

/// Lexer state machine for RQL tokenization.
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize the entire input into a token stream.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, QueryValidationError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }

    /// Consume and return the next token.
    fn next_token(&mut self) -> Result<Token, QueryValidationError> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Ok(Token::Eof);
        }

        let ch = self.input[self.pos];

        match ch {
            // String literals
            '\'' | '"' => self.read_string(),
            // Operators and punctuation
            '=' => {
                self.advance();
                Ok(Token::Eq)
            }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::Neq)
                } else {
                    Err(self.error("Expected '=' after '!'", ValidationErrorKind::SyntaxError))
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::Gte)
                } else {
                    Ok(Token::Gt)
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::Lte)
                } else {
                    Ok(Token::Lt)
                }
            }
            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            // Numbers (may be duration or CIDR component)
            c if c.is_ascii_digit() => self.read_number_or_duration(),
            // Identifiers and keywords
            c if c.is_ascii_alphabetic() || c == '_' => self.read_identifier_or_keyword(),
            _ => Err(self.error(
                &format!("Unexpected character: '{}'", ch),
                ValidationErrorKind::SyntaxError,
            )),
        }
    }

    fn read_string(&mut self) -> Result<Token, QueryValidationError> {
        let quote = self.input[self.pos];
        self.advance(); // consume opening quote

        let mut value = String::new();
        while self.pos < self.input.len() && self.input[self.pos] != quote {
            if self.input[self.pos] == '\\' {
                self.advance();
                if self.pos >= self.input.len() {
                    return Err(self.error(
                        "Unexpected end of string after escape",
                        ValidationErrorKind::UnclosedString,
                    ));
                }
                match self.input[self.pos] {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    '\\' => value.push('\\'),
                    c if c == quote => value.push(c),
                    c => value.push(c),
                }
            } else {
                value.push(self.input[self.pos]);
            }
            self.advance();
        }

        if self.pos >= self.input.len() {
            return Err(self.error("Unclosed string literal", ValidationErrorKind::UnclosedString));
        }

        self.advance(); // consume closing quote

        // Check if this looks like a CIDR notation
        if is_cidr(&value) {
            Ok(Token::Cidr(value))
        } else {
            Ok(Token::StringLiteral(value))
        }
    }

    fn read_number_or_duration(&mut self) -> Result<Token, QueryValidationError> {
        let start = self.pos;
        let mut has_dot = false;

        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let num_str: String = self.input[start..self.pos].iter().collect();

        // Check for duration suffix
        if self.pos < self.input.len() {
            let suffix = self.input[self.pos];
            match suffix {
                's' | 'm' | 'h' | 'd' if !has_dot => {
                    self.advance();
                    let amount: u64 = num_str.parse().map_err(|_| {
                        self.error("Invalid duration number", ValidationErrorKind::InvalidDuration)
                    })?;
                    let unit = match suffix {
                        's' => DurationUnit::Seconds,
                        'm' => DurationUnit::Minutes,
                        'h' => DurationUnit::Hours,
                        'd' => DurationUnit::Days,
                        _ => unreachable!(),
                    };
                    return Ok(Token::Duration(DurationValue { amount, unit }));
                }
                _ => {}
            }
        }

        if has_dot {
            if let Ok(f) = num_str.parse::<f64>() {
                return Ok(Token::NumberLiteral(f));
            }
        }

        if let Ok(i) = num_str.parse::<i64>() {
            Ok(Token::IntegerLiteral(i))
        } else if let Ok(f) = num_str.parse::<f64>() {
            Ok(Token::NumberLiteral(f))
        } else {
            Err(self.error(
                &format!("Invalid number: '{}'", num_str),
                ValidationErrorKind::InvalidLiteral,
            ))
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Result<Token, QueryValidationError> {
        let start = self.pos;
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let word: String = self.input[start..self.pos].iter().collect();

        let token = match word.to_lowercase().as_str() {
            "events" => Token::Events,
            "alerts" => Token::Alerts,
            "agents" => Token::Agents,
            "network" => Token::Network,
            "where" => Token::Where,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "in" => Token::In,
            "between" => Token::Between,
            "contains" => Token::Contains,
            "matches" => Token::Matches,
            "count" => Token::Count,
            "group_by" => Token::GroupBy,
            "time_range" => Token::TimeRange,
            "order_by" => Token::OrderBy,
            "limit" => Token::Limit,
            "last" => Token::Last,
            "asc" => Token::Asc,
            "desc" => Token::Desc,
            _ => Token::Identifier(word),
        };

        Ok(token)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                ' ' | '\t' | '\r' => {
                    self.column += 1;
                    self.pos += 1;
                }
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.pos += 1;
                }
                _ => break,
            }
        }
    }

    fn advance(&mut self) {
        if self.pos < self.input.len() {
            if self.input[self.pos] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn error(&self, message: &str, kind: ValidationErrorKind) -> QueryValidationError {
        QueryValidationError {
            message: message.to_string(),
            position: self.pos,
            line: self.line,
            column: self.column,
            kind,
        }
    }
}

/// Check if a string looks like CIDR notation (e.g., "10.0.0.0/8").
fn is_cidr(s: &str) -> bool {
    if let Some(slash_pos) = s.find('/') {
        let ip_part = &s[..slash_pos];
        let prefix_part = &s[slash_pos + 1..];
        let prefix_ok = prefix_part.parse::<u8>().map_or(false, |p| p <= 128);
        let ip_ok = ip_part.parse::<std::net::IpAddr>().is_ok();
        prefix_ok && ip_ok
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_query_tokenization() {
        let mut lexer = Lexer::new("events where severity = 'critical'");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Events,
                Token::Where,
                Token::Identifier("severity".to_string()),
                Token::Eq,
                Token::StringLiteral("critical".to_string()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_duration_tokenization() {
        let mut lexer = Lexer::new("time_range last 24h");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::TimeRange,
                Token::Last,
                Token::Duration(DurationValue {
                    amount: 24,
                    unit: DurationUnit::Hours
                }),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new(">= <= != > < =");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![Token::Gte, Token::Lte, Token::Neq, Token::Gt, Token::Lt, Token::Eq, Token::Eof]
        );
    }

    #[test]
    fn test_cidr_in_string() {
        let mut lexer = Lexer::new("'10.0.0.0/8'");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Cidr("10.0.0.0/8".to_string()), Token::Eof]);
    }

    #[test]
    fn test_complex_query() {
        let mut lexer = Lexer::new(
            "network where dst_port = 443 and bytes_out > 1000000 time_range last 1h order_by bytes_out desc limit 50",
        );
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::Network));
        assert!(tokens.contains(&Token::And));
        assert!(tokens.contains(&Token::IntegerLiteral(443)));
        assert!(tokens.contains(&Token::IntegerLiteral(1000000)));
        assert!(tokens.contains(&Token::Duration(DurationValue {
            amount: 1,
            unit: DurationUnit::Hours,
        })));
        assert!(tokens.contains(&Token::Desc));
        assert!(tokens.contains(&Token::IntegerLiteral(50)));
    }

    #[test]
    fn test_unclosed_string_error() {
        let mut lexer = Lexer::new("'unclosed string");
        let result = lexer.tokenize();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, ValidationErrorKind::UnclosedString);
    }
}
