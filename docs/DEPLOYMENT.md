# TinyStore Deployment Guide

This guide covers various deployment methods for TinyStore.

## Table of Contents

- [Local Development](#local-development)
- [Docker Deployment](#docker-deployment)
- [Docker Compose](#docker-compose)
- [Production Deployment](#production-deployment)
- [Kubernetes](#kubernetes)
- [Systemd Service](#systemd-service)
- [Configuration](#configuration)
- [Security Considerations](#security-considerations)

## Local Development

### Prerequisites

- Rust 1.75 or later
- Cargo

### Quick Start

```bash
# Clone the repository
git clone https://github.com/phumiphatauk/tinystore
cd tinystore

# Copy example config
cp config/config.example.yaml config/config.yaml

# Edit config as needed
vim config/config.yaml

# Build and run
cargo run --release -- serve
```

The server will start on `http://localhost:9000`.

### Using Makefile

```bash
# Build the project
make build

# Run in development mode
make run

# Run in release mode
make run-release

# Run tests
make test
```

## Docker Deployment

### Build Docker Image

```bash
# Build the image
docker build -t tinystore:latest .

# Or with a specific tag
docker build -t tinystore:0.1.0 .
```

### Run Container

```bash
# Create config and data directories
mkdir -p ./config ./data

# Copy example config
cp config/config.example.yaml ./config/config.yaml

# Edit config
vim ./config/config.yaml

# Run container
docker run -d \
  --name tinystore \
  -p 9000:9000 \
  -v $(pwd)/data:/data \
  -v $(pwd)/config/config.yaml:/config/config.yaml:ro \
  -e RUST_LOG=info \
  tinystore:latest
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Logging level (trace, debug, info, warn, error) |
| `TINYSTORE_CONFIG` | `/config/config.yaml` | Path to config file |
| `TINYSTORE_DATA_DIR` | `/data` | Path to data directory |

## Docker Compose

### Basic Setup

```bash
# Copy example config
cp config/config.example.yaml config/config.yaml

# Edit config
vim config/config.yaml

# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Custom docker-compose.yaml

```yaml
version: '3.8'

services:
  tinystore:
    image: tinystore:latest
    container_name: tinystore
    restart: unless-stopped
    ports:
      - "9000:9000"
    volumes:
      - tinystore-data:/data
      - ./config/config.yaml:/config/config.yaml:ro
    environment:
      - RUST_LOG=info
    networks:
      - backend
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/health"]
      interval: 30s
      timeout: 3s
      retries: 3

networks:
  backend:
    driver: bridge

volumes:
  tinystore-data:
    driver: local
```

## Production Deployment

### Prerequisites

1. **Hardware Requirements** (minimum):
   - CPU: 1 core
   - RAM: 512MB
   - Disk: Depends on data size

2. **Recommended**:
   - CPU: 2+ cores
   - RAM: 2GB+
   - Disk: SSD with adequate space

### Using Pre-built Binary

```bash
# Download latest release
wget https://github.com/phumiphatauk/tinystore/releases/latest/download/tinystore-linux-x86_64

# Make executable
chmod +x tinystore-linux-x86_64
mv tinystore-linux-x86_64 /usr/local/bin/tinystore

# Create config directory
mkdir -p /etc/tinystore
cp config/config.example.yaml /etc/tinystore/config.yaml

# Edit config
vim /etc/tinystore/config.yaml

# Create data directory
mkdir -p /var/lib/tinystore
chown tinystore:tinystore /var/lib/tinystore

# Run
tinystore serve --config /etc/tinystore/config.yaml
```

### Building from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/phumiphatauk/tinystore
cd tinystore
cargo build --release

# Copy binary
sudo cp target/release/tinystore /usr/local/bin/

# Set up config and data
sudo mkdir -p /etc/tinystore /var/lib/tinystore
sudo cp config/config.example.yaml /etc/tinystore/config.yaml
```

## Kubernetes

### Basic Deployment

```yaml
# tinystore-deployment.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: tinystore-config
data:
  config.yaml: |
    server:
      host: "0.0.0.0"
      port: 9000
    storage:
      backend: "filesystem"
      data_dir: "/data"
    auth:
      enabled: true
      credentials:
        - access_key: "tinystore"
          secret_key: "tinystore123"
          admin: true
    region: "us-east-1"
    ui:
      enabled: true
    logging:
      level: "info"
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: tinystore-data
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: tinystore
spec:
  replicas: 1
  selector:
    matchLabels:
      app: tinystore
  template:
    metadata:
      labels:
        app: tinystore
    spec:
      containers:
      - name: tinystore
        image: tinystore:latest
        imagePullPolicy: IfNotPresent
        ports:
        - containerPort: 9000
          name: http
        volumeMounts:
        - name: data
          mountPath: /data
        - name: config
          mountPath: /config/config.yaml
          subPath: config.yaml
        env:
        - name: RUST_LOG
          value: "info"
        - name: TINYSTORE_CONFIG
          value: "/config/config.yaml"
        livenessProbe:
          httpGet:
            path: /health
            port: 9000
          initialDelaySeconds: 5
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 9000
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: tinystore-data
      - name: config
        configMap:
          name: tinystore-config
---
apiVersion: v1
kind: Service
metadata:
  name: tinystore
spec:
  selector:
    app: tinystore
  ports:
  - port: 9000
    targetPort: 9000
    name: http
  type: LoadBalancer
```

### Deploy to Kubernetes

```bash
# Apply configuration
kubectl apply -f tinystore-deployment.yaml

# Check status
kubectl get pods -l app=tinystore
kubectl get svc tinystore

# View logs
kubectl logs -f deployment/tinystore

# Port forward for local access
kubectl port-forward svc/tinystore 9000:9000
```

## Systemd Service

### Create Service File

```bash
sudo vim /etc/systemd/system/tinystore.service
```

```ini
[Unit]
Description=TinyStore - S3-compatible object storage server
After=network.target

[Service]
Type=simple
User=tinystore
Group=tinystore
WorkingDirectory=/var/lib/tinystore
ExecStart=/usr/local/bin/tinystore serve --config /etc/tinystore/config.yaml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=tinystore

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/tinystore
CapabilityBoundingSet=CAP_NET_BIND_SERVICE

# Resource limits
LimitNOFILE=65536
LimitNPROC=512
MemoryLimit=2G

[Install]
WantedBy=multi-user.target
```

### Manage Service

```bash
# Create user
sudo useradd -r -s /bin/false tinystore

# Set permissions
sudo chown -R tinystore:tinystore /var/lib/tinystore
sudo chown -R tinystore:tinystore /etc/tinystore

# Reload systemd
sudo systemctl daemon-reload

# Enable and start service
sudo systemctl enable tinystore
sudo systemctl start tinystore

# Check status
sudo systemctl status tinystore

# View logs
sudo journalctl -u tinystore -f

# Restart service
sudo systemctl restart tinystore

# Stop service
sudo systemctl stop tinystore
```

## Configuration

### Production Configuration Example

```yaml
server:
  host: "0.0.0.0"
  port: 9000

storage:
  backend: "filesystem"
  data_dir: "/var/lib/tinystore/data"
  max_object_size: "5GB"

auth:
  enabled: true
  credentials:
    - access_key: "${TINYSTORE_ACCESS_KEY}"
      secret_key: "${TINYSTORE_SECRET_KEY}"
      admin: true

region: "us-east-1"

ui:
  enabled: true
  path: "/ui"

logging:
  level: "info"
  format: "json"
  file: "/var/log/tinystore/tinystore.log"

limits:
  max_bucket_count: 100
  max_keys_per_list: 1000
  max_multipart_parts: 10000
  max_request_size: "100MB"

metrics:
  enabled: true
  path: "/metrics"
```

### Environment Variable Substitution

TinyStore supports environment variable substitution in config files:

```bash
export TINYSTORE_ACCESS_KEY="my-access-key"
export TINYSTORE_SECRET_KEY="my-secret-key"
tinystore serve --config /etc/tinystore/config.yaml
```

## Security Considerations

### 1. Credentials

- **Never use default credentials in production**
- Use strong, randomly generated access and secret keys
- Store credentials in environment variables or secrets management systems
- Rotate credentials regularly

```bash
# Generate secure credentials
ACCESS_KEY=$(openssl rand -hex 16)
SECRET_KEY=$(openssl rand -hex 32)
```

### 2. Network Security

- Run behind a reverse proxy (nginx, Traefik) for TLS termination
- Use firewall rules to restrict access
- Consider VPN or private networks for sensitive data

### 3. TLS/HTTPS

Example nginx configuration:

```nginx
server {
    listen 443 ssl http2;
    server_name s3.example.com;

    ssl_certificate /etc/ssl/certs/tinystore.crt;
    ssl_certificate_key /etc/ssl/private/tinystore.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    location / {
        proxy_pass http://localhost:9000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Large file uploads
        client_max_body_size 5G;
        proxy_request_buffering off;
    }
}
```

### 4. File Permissions

```bash
# Set strict permissions
chmod 700 /var/lib/tinystore
chmod 600 /etc/tinystore/config.yaml
chown -R tinystore:tinystore /var/lib/tinystore
chown -R tinystore:tinystore /etc/tinystore
```

### 5. Resource Limits

Configure appropriate resource limits in your deployment:

- Memory limits to prevent OOM
- Connection limits to prevent exhaustion
- Request size limits to prevent abuse
- Rate limiting (via reverse proxy)

### 6. Monitoring

Set up monitoring for:
- Disk usage
- Memory usage
- Request rates
- Error rates
- Authentication failures

### 7. Backups

Implement regular backups:

```bash
# Simple backup script
#!/bin/bash
BACKUP_DIR=/backups/tinystore
DATE=$(date +%Y%m%d-%H%M%S)
tar -czf $BACKUP_DIR/tinystore-$DATE.tar.gz /var/lib/tinystore/data

# Keep only last 7 days
find $BACKUP_DIR -name "tinystore-*.tar.gz" -mtime +7 -delete
```

## Monitoring

### Health Check

```bash
curl http://localhost:9000/health
```

### Metrics (if enabled)

```bash
curl http://localhost:9000/metrics
```

### Logging

Configure structured logging for production:

```yaml
logging:
  level: "info"
  format: "json"
  file: "/var/log/tinystore/tinystore.log"
```

View logs:
```bash
# Systemd
sudo journalctl -u tinystore -f

# Docker
docker logs -f tinystore

# File
tail -f /var/log/tinystore/tinystore.log | jq
```

## Troubleshooting

### Permission Denied

```bash
# Check file permissions
ls -la /var/lib/tinystore
ls -la /etc/tinystore

# Fix permissions
sudo chown -R tinystore:tinystore /var/lib/tinystore
sudo chmod 700 /var/lib/tinystore
```

### Port Already in Use

```bash
# Check what's using the port
sudo lsof -i :9000

# Change port in config.yaml
server:
  port: 9001
```

### Out of Memory

```bash
# Check memory usage
free -h

# Increase memory limit in systemd service
MemoryLimit=4G

# Or reduce concurrent connections in config
```

### Connection Refused

```bash
# Check if service is running
systemctl status tinystore

# Check firewall
sudo ufw status
sudo ufw allow 9000/tcp

# Check logs
journalctl -u tinystore -n 100
```

## Performance Tuning

### 1. File System

- Use SSD for better I/O performance
- Consider using XFS or ext4 for large files
- Mount with `noatime` option

### 2. Kernel Parameters

```bash
# /etc/sysctl.conf
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 8192
fs.file-max = 2097152
```

### 3. Ulimit

```bash
# /etc/security/limits.conf
tinystore soft nofile 65536
tinystore hard nofile 65536
```

## Upgrade

### Docker

```bash
# Pull new image
docker pull tinystore:latest

# Stop and remove old container
docker stop tinystore
docker rm tinystore

# Run new container
docker run -d \
  --name tinystore \
  -p 9000:9000 \
  -v $(pwd)/data:/data \
  -v $(pwd)/config/config.yaml:/config/config.yaml:ro \
  tinystore:latest
```

### Systemd

```bash
# Stop service
sudo systemctl stop tinystore

# Backup data
sudo tar -czf /tmp/tinystore-backup.tar.gz /var/lib/tinystore

# Update binary
sudo cp target/release/tinystore /usr/local/bin/

# Start service
sudo systemctl start tinystore

# Check status
sudo systemctl status tinystore
```

## See Also

- [API Documentation](API.md)
- [Usage Examples](EXAMPLES.md)
- [Configuration Reference](../config/config.example.yaml)
