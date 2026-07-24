//! Integration test suite for Raksha Portal API.
//!
//! These tests spin up a real HTTP server with a test database
//! and exercise the API endpoints end-to-end.
//!
//! # Requirements
//! - PostgreSQL running with `raksha_test` database
//! - Redis running on localhost:6379
//! - Migrations applied (handled automatically by test harness)
//!
//! # Running
//! ```sh
//! DATABASE_URL=postgres://raksha:test_secret@localhost:5432/raksha_test cargo test --test integration
//! ```

mod common;
mod auth_test;
mod user_test;
