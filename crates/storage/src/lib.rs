//! Storage backend implementations for TinyStore

pub mod backend;
pub mod filesystem;
pub mod memory;

pub use backend::{
    StorageBackend, ListObjectsParams, GetObjectResult, PutObjectResult,
    CopyObjectResult, CompleteMultipartResult, PartInfo, CompletedPart,
};
pub use filesystem::FilesystemBackend;
pub use memory::MemoryBackend;
