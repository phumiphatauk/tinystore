//! Shared types and utilities for TinyStore
//!
//! This crate contains types that are shared between the frontend (UI) and backend.

pub mod bucket;
pub mod object;
pub mod error;
pub mod api;

pub use bucket::{BucketInfo, BucketStats};
pub use object::{ObjectInfo, ObjectMetadata};
pub use error::{StorageError, StorageResult};
