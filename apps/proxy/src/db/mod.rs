//! Database module
//!
//! This module provides database connection management, queries, and related utilities
//! for the AI Analytics Dashboard.

pub mod connection;
pub mod queries;
pub mod seeds;

pub use connection::{DatabaseConfig, DatabaseManager, PoolStats};
pub use queries::{CompanyQueries, AiClientQueries, AiProviderQueries, RequestEventQueries};
pub use seeds::seed_database;