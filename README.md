# TinyStore

> A tiny, fast, and efficient S3-compatible object storage server

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## Overview

TinyStore is a lightweight, S3-compatible object storage server written in Rust. Designed for minimal resource usage (<30MB RAM idle), it's perfect for:

- Local development and testing
- CI/CD pipelines
- Edge computing and IoT devices
- Small-scale production deployments

## Features

- **Lightweight**: <30MB RAM at idle, <100MB under moderate load
- **S3 Compatible**: Works with AWS SDKs (boto3, aws-sdk-js, aws-sdk-go, etc.)
- **Single Binary**: Easy deployment with no external dependencies
- **Web UI**: Built-in management interface using Leptos (full-stack Rust)
- **Fast**: High-performance storage with efficient async I/O

## Quick Start

### Using Cargo

```bash
# Clone the repository
git clone https://github.com/phumiphatauk/tinystore
cd tinystore

# Build the project
cargo build --release

# Run the server
./target/release/tinystore serve
```

### Using Docker

```bash
docker run -p 9000:9000 -v ./data:/data phumiphatauk/tinystore
```

### Configuration

Create a `config.yaml` file (see `config/config.example.yaml` for reference):

```yaml
server:
  host: "0.0.0.0"
  port: 9000

storage:
  backend: "filesystem"
  data_dir: "./data"

auth:
  enabled: true
  credentials:
    - access_key: "tinystore"
      secret_key: "tinystore123"
      admin: true
```

## Development Status

This project is currently under active development. The workspace structure is complete, and we're implementing features step by step:

- [x] Step 1: Workspace structure setup
- [x] Step 2: Shared types implementation
- [x] Step 3: Storage backend implementation
- [ ] Step 4: S3 API handlers
- [ ] Step 5: Authentication
- [ ] Step 6: Error handling
- [ ] Step 7: Leptos UI setup
- [ ] Step 8: UI pages implementation
- [ ] Step 9: Testing
- [ ] Step 10: Docker and documentation

## Project Structure

```
tinystore/
├── crates/
│   ├── shared/       # Shared types between frontend/backend
│   ├── storage/      # Storage backend implementations
│   ├── auth/         # Authentication (AWS Signature V4)
│   ├── s3-api/       # S3-compatible API handlers
│   ├── ui/           # Leptos web UI
│   └── server/       # Main binary
├── config/           # Configuration files
├── tests/            # Integration and compatibility tests
└── public/           # Static assets for UI
```

## Supported S3 Operations

### Phase 1 (MVP)
- Bucket operations: Create, Delete, List, Head
- Object operations: Put, Get, Head, Delete, List, Copy

### Phase 2 (Essential)
- Multipart uploads
- Bulk delete

### Phase 3 (Advanced)
- Pre-signed URLs
- Object versioning
- Lifecycle policies

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

**phumiphatauk**

- GitHub: [@phumiphatauk](https://github.com/phumiphatauk)
