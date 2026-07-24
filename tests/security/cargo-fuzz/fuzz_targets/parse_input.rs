//! Fuzz target: Input parsing
//!
//! This target exercises Raksha's input parsing and deserialization paths
//! with arbitrary byte sequences to find panics, memory issues, and logic bugs.
//!
//! Run with:
//!   cargo fuzz run parse_input -- -max_total_time=300
//!
//! Run with sanitizer:
//!   cargo fuzz run parse_input --sanitizer=address

#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

/// Simulated input event structure matching Raksha's expected input format.
/// Replace with actual types from raksha-core once available.
#[derive(Debug, Arbitrary)]
struct SecurityEvent {
    /// Source IP address (may be malformed)
    source_ip: String,
    /// Destination IP address (may be malformed)  
    dest_ip: String,
    /// Port number
    port: u16,
    /// Protocol identifier
    protocol: u8,
    /// Raw payload bytes
    payload: Vec<u8>,
    /// Timestamp (unix epoch)
    timestamp: u64,
    /// Event severity (0-10)
    severity: u8,
    /// Rule ID that triggered this event
    rule_id: Option<String>,
}

/// Simulated JSON-like input that exercises deserialization paths
#[derive(Debug, Arbitrary)]
struct RawInput {
    content_type: ContentType,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
}

#[derive(Debug, Arbitrary)]
enum ContentType {
    Json,
    Yaml,
    Toml,
    Binary,
    Unknown,
}

fuzz_target!(|data: &[u8]| {
    // Target 1: Raw byte parsing
    // Simulates receiving arbitrary network data
    fuzz_raw_bytes(data);

    // Target 2: Structured input via Arbitrary
    if let Ok(event) = arbitrary::Unstructured::new(data).and_then(SecurityEvent::arbitrary) {
        fuzz_security_event(event);
    }

    // Target 3: Raw input deserialization
    if let Ok(input) = arbitrary::Unstructured::new(data).and_then(RawInput::arbitrary) {
        fuzz_raw_input(input);
    }
});

/// Fuzz raw byte parsing paths.
/// Tests that no combination of bytes causes a panic.
fn fuzz_raw_bytes(data: &[u8]) {
    // Exercise IP address parsing
    if data.len() >= 4 {
        let ip_str = format!("{}.{}.{}.{}", data[0], data[1], data[2], data[3]);
        let _ = ip_str.parse::<std::net::Ipv4Addr>();
    }

    // Exercise string parsing from potentially invalid UTF-8
    let _ = std::str::from_utf8(data);

    // Exercise bounded slice operations (should never panic)
    if data.len() > 8 {
        let _header = &data[..8];
        let _body = &data[8..];
    }

    // Exercise JSON-like parsing on raw bytes
    if let Ok(s) = std::str::from_utf8(data) {
        // Simulate parsing a JSON security event
        // Replace with actual parser: raksha_parser::parse_event(s);
        let _ = parse_key_value(s);
    }
}

/// Fuzz structured SecurityEvent processing.
/// Tests validation and normalization logic.
fn fuzz_security_event(event: SecurityEvent) {
    // Validate IP format handling
    let _ = event.source_ip.parse::<std::net::IpAddr>();
    let _ = event.dest_ip.parse::<std::net::IpAddr>();

    // Validate severity bounds
    let _normalized_severity = event.severity.min(10);

    // Validate payload size limits
    let _truncated = if event.payload.len() > 65535 {
        &event.payload[..65535]
    } else {
        &event.payload
    };

    // Validate rule_id format if present
    if let Some(ref rule_id) = event.rule_id {
        let _ = validate_rule_id(rule_id);
    }
}

/// Fuzz raw HTTP-like input deserialization.
fn fuzz_raw_input(input: RawInput) {
    match input.content_type {
        ContentType::Json => {
            // Simulate JSON deserialization
            if let Ok(s) = std::str::from_utf8(&input.body) {
                let _ = parse_key_value(s);
            }
        }
        ContentType::Yaml | ContentType::Toml => {
            // Simulate config deserialization
            if let Ok(s) = std::str::from_utf8(&input.body) {
                // Replace with: raksha_config::parse(s, format);
                let _ = s.lines().count();
            }
        }
        ContentType::Binary => {
            // Simulate binary protocol parsing
            if input.body.len() >= 4 {
                let _version = u16::from_le_bytes([input.body[0], input.body[1]]);
                let _length = u16::from_le_bytes([input.body[2], input.body[3]]);
            }
        }
        ContentType::Unknown => {
            // Content-type sniffing should not panic
            let _ = guess_content_type(&input.body);
        }
    }

    // Header validation should handle arbitrary strings
    for (key, value) in &input.headers {
        let _ = validate_header(key, value);
    }
}

// --- Helper functions (replace with actual Raksha implementations) ---

/// Simple key-value parser that exercises string operations
fn parse_key_value(input: &str) -> Vec<(&str, &str)> {
    input
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect()
}

/// Validate a rule ID matches expected format: raksha-XX-NNN
fn validate_rule_id(id: &str) -> bool {
    if id.len() < 10 || id.len() > 64 {
        return false;
    }
    id.starts_with("raksha-")
        && id.chars().all(|c| c.is_alphanumeric() || c == '-')
}

/// Guess content type from magic bytes
fn guess_content_type(data: &[u8]) -> &'static str {
    match data.get(..4) {
        Some(b"\x7fELF") => "application/x-elf",
        Some(b"\x89PNG") => "image/png",
        Some(b"PK\x03\x04") => "application/zip",
        Some(b) if b.starts_with(b"{") => "application/json",
        Some(b) if b.starts_with(b"<") => "application/xml",
        _ => "application/octet-stream",
    }
}

/// Validate header key/value (no panics on arbitrary input)
fn validate_header(key: &str, value: &str) -> bool {
    !key.is_empty()
        && key.len() <= 256
        && value.len() <= 8192
        && key.is_ascii()
        && !value.contains('\0')
}
