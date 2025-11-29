# TinyStore Tests

This directory contains comprehensive tests for TinyStore.

## Test Structure

```
tests/
├── integration/     # Rust integration tests
└── compatibility/   # S3 compatibility tests (Python/boto3)
```

## Running Tests

### Unit Tests

Unit tests are embedded in each crate. Run all unit tests with:

```bash
cargo test --lib
```

Or test specific crates:

```bash
cargo test -p tinystore-storage
cargo test -p tinystore-auth
```

### Integration Tests

Integration tests are in the `tests/integration/` directory:

```bash
cargo test --test '*'
```

Or run specific integration tests:

```bash
cargo test --test s3_api_tests
```

### Compatibility Tests

The compatibility tests verify TinyStore works with standard S3 clients like boto3.

#### Prerequisites

Install Python dependencies:

```bash
pip install boto3 pytest
```

#### Running Compatibility Tests

1. Start TinyStore server:

```bash
cargo run --release -- serve
```

2. In another terminal, run the compatibility tests:

```bash
pytest tests/compatibility/test_s3_compatibility.py -v
```

Or run specific tests:

```bash
pytest tests/compatibility/test_s3_compatibility.py::test_put_and_get_object -v
```

## Test Coverage

### Storage Backend Tests
- ✅ Bucket operations (create, delete, list, exists)
- ✅ Object operations (put, get, delete, head)
- ✅ Range requests
- ✅ Listing with prefix/delimiter
- ✅ Copy operations
- ✅ Multipart uploads
- ✅ Validation (bucket names, object keys)
- ✅ Error handling

### Authentication Tests
- ✅ Credential management
- ✅ AWS Signature V4 calculation
- ✅ Signature verification
- ✅ Header canonicalization
- ✅ Query string canonicalization

### Integration Tests
- ✅ Complete storage workflows
- ✅ Multipart upload flows
- ✅ Directory listing
- ✅ Object copying
- ✅ Error scenarios

### Compatibility Tests
- ✅ boto3 S3 client compatibility
- ✅ Bucket operations
- ✅ Object CRUD operations
- ✅ Multipart uploads
- ✅ Range requests
- ✅ List operations with filters

## Continuous Integration

Run all tests:

```bash
# Run all unit and integration tests
cargo test --all

# Run compatibility tests (requires server)
./scripts/run-compat-tests.sh
```

## Writing New Tests

### Adding Unit Tests

Add tests directly in the source files using `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_feature() {
        // Test code here
    }
}
```

### Adding Integration Tests

Create a new file in `tests/integration/`:

```rust
//! Integration test for new feature

use tinystore_storage::MemoryBackend;

#[tokio::test]
async fn test_new_feature_integration() {
    // Test code here
}
```

### Adding Compatibility Tests

Add new test functions to `tests/compatibility/test_s3_compatibility.py`:

```python
def test_new_s3_feature(s3_client, test_bucket):
    """Test description."""
    # Test code here
    assert result is expected
```
