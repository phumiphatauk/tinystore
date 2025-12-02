//! S3-compatible API implementation

pub mod router;
pub mod bucket;
pub mod object;
pub mod multipart;
pub mod xml;
pub mod error;
pub mod health;

pub use router::create_s3_router;
pub use error::{S3Error, S3Result};
pub use health::health_check;
