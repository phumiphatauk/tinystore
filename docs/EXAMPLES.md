# TinyStore Usage Examples

This guide provides practical examples for using TinyStore with various tools and SDKs.

## Table of Contents

- [AWS CLI](#aws-cli)
- [Python (boto3)](#python-boto3)
- [JavaScript/TypeScript](#javascripttypescript)
- [Go](#go)
- [Rust](#rust)
- [MinIO Client](#minio-client)
- [curl](#curl)
- [Integration Examples](#integration-examples)

## AWS CLI

### Setup

```bash
# Install AWS CLI
pip install awscli

# Configure credentials
aws configure set aws_access_key_id tinystore
aws configure set aws_secret_access_key tinystore123
aws configure set default.region us-east-1

# Set endpoint for all commands
export AWS_ENDPOINT_URL=http://localhost:9000
```

### Bucket Operations

```bash
# Create bucket
aws s3 mb s3://my-bucket --endpoint-url $AWS_ENDPOINT_URL

# List buckets
aws s3 ls --endpoint-url $AWS_ENDPOINT_URL

# Delete bucket
aws s3 rb s3://my-bucket --endpoint-url $AWS_ENDPOINT_URL

# Delete bucket with all contents
aws s3 rb s3://my-bucket --force --endpoint-url $AWS_ENDPOINT_URL
```

### Object Operations

```bash
# Upload file
aws s3 cp file.txt s3://my-bucket/ --endpoint-url $AWS_ENDPOINT_URL

# Upload directory
aws s3 cp ./my-dir s3://my-bucket/my-dir --recursive --endpoint-url $AWS_ENDPOINT_URL

# Download file
aws s3 cp s3://my-bucket/file.txt . --endpoint-url $AWS_ENDPOINT_URL

# Download directory
aws s3 cp s3://my-bucket/my-dir ./my-dir --recursive --endpoint-url $AWS_ENDPOINT_URL

# List objects
aws s3 ls s3://my-bucket/ --endpoint-url $AWS_ENDPOINT_URL

# List with prefix
aws s3 ls s3://my-bucket/prefix/ --endpoint-url $AWS_ENDPOINT_URL

# Delete object
aws s3 rm s3://my-bucket/file.txt --endpoint-url $AWS_ENDPOINT_URL

# Delete all objects in bucket
aws s3 rm s3://my-bucket --recursive --endpoint-url $AWS_ENDPOINT_URL

# Copy object
aws s3 cp s3://my-bucket/file.txt s3://my-bucket/copy.txt --endpoint-url $AWS_ENDPOINT_URL

# Move object
aws s3 mv s3://my-bucket/file.txt s3://my-bucket/moved.txt --endpoint-url $AWS_ENDPOINT_URL

# Sync directory
aws s3 sync ./local-dir s3://my-bucket/remote-dir --endpoint-url $AWS_ENDPOINT_URL
```

### Advanced Operations

```bash
# Get object metadata
aws s3api head-object \
  --bucket my-bucket \
  --key file.txt \
  --endpoint-url $AWS_ENDPOINT_URL

# List objects with details
aws s3api list-objects-v2 \
  --bucket my-bucket \
  --prefix docs/ \
  --max-keys 100 \
  --endpoint-url $AWS_ENDPOINT_URL

# Upload with metadata
aws s3api put-object \
  --bucket my-bucket \
  --key file.txt \
  --body file.txt \
  --content-type "text/plain" \
  --endpoint-url $AWS_ENDPOINT_URL

# Delete multiple objects
aws s3api delete-objects \
  --bucket my-bucket \
  --delete '{"Objects":[{"Key":"file1.txt"},{"Key":"file2.txt"}]}' \
  --endpoint-url $AWS_ENDPOINT_URL
```

## Python (boto3)

### Installation

```bash
pip install boto3
```

### Basic Usage

```python
import boto3
from botocore.client import Config

# Create S3 client
s3 = boto3.client(
    's3',
    endpoint_url='http://localhost:9000',
    aws_access_key_id='tinystore',
    aws_secret_access_key='tinystore123',
    region_name='us-east-1',
    config=Config(signature_version='s3v4')
)

# Create bucket
s3.create_bucket(Bucket='my-bucket')

# List buckets
response = s3.list_buckets()
for bucket in response['Buckets']:
    print(f"Bucket: {bucket['Name']}")

# Upload file
s3.upload_file('local-file.txt', 'my-bucket', 'remote-file.txt')

# Upload with metadata
s3.upload_file(
    'local-file.txt',
    'my-bucket',
    'remote-file.txt',
    ExtraArgs={
        'ContentType': 'text/plain',
        'Metadata': {
            'author': 'John Doe',
            'version': '1.0'
        }
    }
)

# Download file
s3.download_file('my-bucket', 'remote-file.txt', 'local-file.txt')

# List objects
response = s3.list_objects_v2(Bucket='my-bucket')
for obj in response.get('Contents', []):
    print(f"Object: {obj['Key']} (Size: {obj['Size']} bytes)")

# Delete object
s3.delete_object(Bucket='my-bucket', Key='remote-file.txt')

# Delete bucket
s3.delete_bucket(Bucket='my-bucket')
```

### Advanced Examples

```python
# Upload large file with multipart upload
from boto3.s3.transfer import TransferConfig

config = TransferConfig(
    multipart_threshold=1024 * 25,  # 25MB
    max_concurrency=10,
    multipart_chunksize=1024 * 25,
    use_threads=True
)

s3.upload_file(
    'large-file.bin',
    'my-bucket',
    'large-file.bin',
    Config=config
)

# Upload from memory
import io

data = b"Hello, World!"
s3.put_object(
    Bucket='my-bucket',
    Key='greeting.txt',
    Body=io.BytesIO(data)
)

# Download to memory
obj = s3.get_object(Bucket='my-bucket', Key='greeting.txt')
content = obj['Body'].read()
print(content.decode('utf-8'))

# Generate presigned URL (if supported)
url = s3.generate_presigned_url(
    'get_object',
    Params={'Bucket': 'my-bucket', 'Key': 'file.txt'},
    ExpiresIn=3600  # 1 hour
)
print(f"Presigned URL: {url}")

# Copy object
s3.copy_object(
    Bucket='my-bucket',
    CopySource={'Bucket': 'my-bucket', 'Key': 'source.txt'},
    Key='destination.txt'
)

# Batch delete
objects_to_delete = [
    {'Key': 'file1.txt'},
    {'Key': 'file2.txt'},
    {'Key': 'file3.txt'}
]

s3.delete_objects(
    Bucket='my-bucket',
    Delete={'Objects': objects_to_delete}
)

# List with pagination
paginator = s3.get_paginator('list_objects_v2')
for page in paginator.paginate(Bucket='my-bucket', Prefix='docs/'):
    for obj in page.get('Contents', []):
        print(f"Object: {obj['Key']}")
```

### Error Handling

```python
from botocore.exceptions import ClientError

try:
    s3.head_bucket(Bucket='my-bucket')
    print("Bucket exists")
except ClientError as e:
    if e.response['Error']['Code'] == '404':
        print("Bucket does not exist")
    else:
        print(f"Error: {e}")

# Check if object exists
def object_exists(bucket, key):
    try:
        s3.head_object(Bucket=bucket, Key=key)
        return True
    except ClientError as e:
        if e.response['Error']['Code'] == '404':
            return False
        raise

if object_exists('my-bucket', 'file.txt'):
    print("Object exists")
else:
    print("Object does not exist")
```

## JavaScript/TypeScript

### Installation

```bash
npm install @aws-sdk/client-s3
```

### Basic Usage (AWS SDK v3)

```javascript
const { S3Client, ListBucketsCommand, CreateBucketCommand,
        PutObjectCommand, GetObjectCommand, DeleteObjectCommand } = require('@aws-sdk/client-s3');
const { Upload } = require('@aws-sdk/lib-storage');
const fs = require('fs');

// Create S3 client
const s3Client = new S3Client({
  endpoint: 'http://localhost:9000',
  region: 'us-east-1',
  credentials: {
    accessKeyId: 'tinystore',
    secretAccessKey: 'tinystore123',
  },
  forcePathStyle: true,
});

// Create bucket
async function createBucket() {
  await s3Client.send(new CreateBucketCommand({
    Bucket: 'my-bucket',
  }));
  console.log('Bucket created');
}

// List buckets
async function listBuckets() {
  const response = await s3Client.send(new ListBucketsCommand({}));
  response.Buckets.forEach(bucket => {
    console.log(`Bucket: ${bucket.Name}`);
  });
}

// Upload object
async function uploadObject() {
  await s3Client.send(new PutObjectCommand({
    Bucket: 'my-bucket',
    Key: 'file.txt',
    Body: 'Hello, World!',
    ContentType: 'text/plain',
  }));
  console.log('Object uploaded');
}

// Upload file
async function uploadFile() {
  const fileStream = fs.createReadStream('local-file.txt');

  const upload = new Upload({
    client: s3Client,
    params: {
      Bucket: 'my-bucket',
      Key: 'remote-file.txt',
      Body: fileStream,
    },
  });

  upload.on('httpUploadProgress', (progress) => {
    console.log(`Uploaded: ${progress.loaded}/${progress.total} bytes`);
  });

  await upload.done();
  console.log('File uploaded');
}

// Download object
async function downloadObject() {
  const response = await s3Client.send(new GetObjectCommand({
    Bucket: 'my-bucket',
    Key: 'file.txt',
  }));

  const content = await streamToString(response.Body);
  console.log('Content:', content);
}

// Helper to convert stream to string
function streamToString(stream) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    stream.on('data', chunk => chunks.push(chunk));
    stream.on('error', reject);
    stream.on('end', () => resolve(Buffer.concat(chunks).toString('utf-8')));
  });
}

// Delete object
async function deleteObject() {
  await s3Client.send(new DeleteObjectCommand({
    Bucket: 'my-bucket',
    Key: 'file.txt',
  }));
  console.log('Object deleted');
}

// Run examples
(async () => {
  await createBucket();
  await listBuckets();
  await uploadObject();
  await uploadFile();
  await downloadObject();
  await deleteObject();
})();
```

### TypeScript Example

```typescript
import { S3Client, PutObjectCommand, GetObjectCommand } from '@aws-sdk/client-s3';
import { Upload } from '@aws-sdk/lib-storage';
import * as fs from 'fs';

const s3Client = new S3Client({
  endpoint: 'http://localhost:9000',
  region: 'us-east-1',
  credentials: {
    accessKeyId: 'tinystore',
    secretAccessKey: 'tinystore123',
  },
  forcePathStyle: true,
});

interface UploadOptions {
  bucket: string;
  key: string;
  filePath: string;
}

async function uploadFile({ bucket, key, filePath }: UploadOptions): Promise<void> {
  const fileStream = fs.createReadStream(filePath);

  const upload = new Upload({
    client: s3Client,
    params: {
      Bucket: bucket,
      Key: key,
      Body: fileStream,
    },
  });

  await upload.done();
  console.log(`Uploaded ${filePath} to ${bucket}/${key}`);
}

async function downloadFile(bucket: string, key: string, destPath: string): Promise<void> {
  const response = await s3Client.send(new GetObjectCommand({
    Bucket: bucket,
    Key: key,
  }));

  const writeStream = fs.createWriteStream(destPath);
  response.Body.pipe(writeStream);

  await new Promise((resolve, reject) => {
    writeStream.on('finish', resolve);
    writeStream.on('error', reject);
  });

  console.log(`Downloaded ${bucket}/${key} to ${destPath}`);
}
```

## Go

### Installation

```bash
go get github.com/aws/aws-sdk-go/aws
go get github.com/aws/aws-sdk-go/aws/credentials
go get github.com/aws/aws-sdk-go/aws/session
go get github.com/aws/aws-sdk-go/service/s3
```

### Basic Usage

```go
package main

import (
    "bytes"
    "fmt"
    "io"
    "log"
    "os"

    "github.com/aws/aws-sdk-go/aws"
    "github.com/aws/aws-sdk-go/aws/credentials"
    "github.com/aws/aws-sdk-go/aws/session"
    "github.com/aws/aws-sdk-go/service/s3"
)

func main() {
    // Create session
    sess, err := session.NewSession(&aws.Config{
        Endpoint:         aws.String("http://localhost:9000"),
        Region:           aws.String("us-east-1"),
        Credentials:      credentials.NewStaticCredentials("tinystore", "tinystore123", ""),
        S3ForcePathStyle: aws.Bool(true),
    })
    if err != nil {
        log.Fatal(err)
    }

    svc := s3.New(sess)

    // Create bucket
    _, err = svc.CreateBucket(&s3.CreateBucketInput{
        Bucket: aws.String("my-bucket"),
    })
    if err != nil {
        log.Fatal(err)
    }
    fmt.Println("Bucket created")

    // List buckets
    result, err := svc.ListBuckets(&s3.ListBucketsInput{})
    if err != nil {
        log.Fatal(err)
    }
    for _, bucket := range result.Buckets {
        fmt.Printf("Bucket: %s\n", *bucket.Name)
    }

    // Upload object
    _, err = svc.PutObject(&s3.PutObjectInput{
        Bucket:      aws.String("my-bucket"),
        Key:         aws.String("file.txt"),
        Body:        bytes.NewReader([]byte("Hello, World!")),
        ContentType: aws.String("text/plain"),
    })
    if err != nil {
        log.Fatal(err)
    }
    fmt.Println("Object uploaded")

    // Upload file
    file, err := os.Open("local-file.txt")
    if err != nil {
        log.Fatal(err)
    }
    defer file.Close()

    _, err = svc.PutObject(&s3.PutObjectInput{
        Bucket: aws.String("my-bucket"),
        Key:    aws.String("remote-file.txt"),
        Body:   file,
    })
    if err != nil {
        log.Fatal(err)
    }
    fmt.Println("File uploaded")

    // Download object
    getResult, err := svc.GetObject(&s3.GetObjectInput{
        Bucket: aws.String("my-bucket"),
        Key:    aws.String("file.txt"),
    })
    if err != nil {
        log.Fatal(err)
    }
    defer getResult.Body.Close()

    body, err := io.ReadAll(getResult.Body)
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Content: %s\n", body)

    // List objects
    listResult, err := svc.ListObjectsV2(&s3.ListObjectsV2Input{
        Bucket: aws.String("my-bucket"),
    })
    if err != nil {
        log.Fatal(err)
    }
    for _, object := range listResult.Contents {
        fmt.Printf("Object: %s (Size: %d bytes)\n", *object.Key, *object.Size)
    }

    // Delete object
    _, err = svc.DeleteObject(&s3.DeleteObjectInput{
        Bucket: aws.String("my-bucket"),
        Key:    aws.String("file.txt"),
    })
    if err != nil {
        log.Fatal(err)
    }
    fmt.Println("Object deleted")
}
```

## Rust

### Cargo.toml

```toml
[dependencies]
aws-config = "1.0"
aws-sdk-s3 = "1.0"
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Error};
use aws_sdk_s3::primitives::ByteStream;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configure client
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env()
        .region(region_provider)
        .endpoint_url("http://localhost:9000")
        .load()
        .await;

    let client = Client::new(&config);

    // Create bucket
    client
        .create_bucket()
        .bucket("my-bucket")
        .send()
        .await?;
    println!("Bucket created");

    // List buckets
    let resp = client.list_buckets().send().await?;
    for bucket in resp.buckets() {
        println!("Bucket: {}", bucket.name().unwrap_or_default());
    }

    // Upload object
    client
        .put_object()
        .bucket("my-bucket")
        .key("file.txt")
        .body(ByteStream::from_static(b"Hello, World!"))
        .send()
        .await?;
    println!("Object uploaded");

    // Download object
    let resp = client
        .get_object()
        .bucket("my-bucket")
        .key("file.txt")
        .send()
        .await?;

    let data = resp.body.collect().await?;
    let content = String::from_utf8(data.to_vec()).unwrap();
    println!("Content: {}", content);

    // Delete object
    client
        .delete_object()
        .bucket("my-bucket")
        .key("file.txt")
        .send()
        .await?;
    println!("Object deleted");

    Ok(())
}
```

## MinIO Client

### Installation

```bash
# Linux/macOS
wget https://dl.min.io/client/mc/release/linux-amd64/mc
chmod +x mc
sudo mv mc /usr/local/bin/
```

### Usage

```bash
# Add TinyStore alias
mc alias set tinystore http://localhost:9000 tinystore tinystore123

# List buckets
mc ls tinystore

# Create bucket
mc mb tinystore/my-bucket

# Upload file
mc cp file.txt tinystore/my-bucket/

# Upload directory
mc cp --recursive ./my-dir tinystore/my-bucket/

# Download file
mc cp tinystore/my-bucket/file.txt .

# List objects
mc ls tinystore/my-bucket

# Delete object
mc rm tinystore/my-bucket/file.txt

# Delete bucket
mc rb tinystore/my-bucket

# Mirror directory
mc mirror ./local-dir tinystore/my-bucket/remote-dir

# Watch for changes
mc watch tinystore/my-bucket

# Get object info
mc stat tinystore/my-bucket/file.txt
```

## curl

### Bucket Operations

```bash
# List buckets (requires AWS signature)
# Use aws-signature-proxy or pre-signed URLs

# Create bucket
curl -X PUT http://localhost:9000/my-bucket \
  -H "Authorization: AWS4-HMAC-SHA256 ..."

# Delete bucket
curl -X DELETE http://localhost:9000/my-bucket \
  -H "Authorization: AWS4-HMAC-SHA256 ..."
```

### Object Operations

```bash
# Upload object
curl -X PUT http://localhost:9000/my-bucket/file.txt \
  -H "Authorization: AWS4-HMAC-SHA256 ..." \
  -H "Content-Type: text/plain" \
  --data-binary "@file.txt"

# Download object
curl http://localhost:9000/my-bucket/file.txt \
  -H "Authorization: AWS4-HMAC-SHA256 ..." \
  -o file.txt

# Delete object
curl -X DELETE http://localhost:9000/my-bucket/file.txt \
  -H "Authorization: AWS4-HMAC-SHA256 ..."
```

### Health Check

```bash
# Health check (no auth required)
curl http://localhost:9000/health
```

## Integration Examples

### Backup Script (Bash)

```bash
#!/bin/bash
set -e

BUCKET="backups"
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_DIR="/var/www"
BACKUP_FILE="backup-$DATE.tar.gz"

# Create archive
tar -czf /tmp/$BACKUP_FILE $BACKUP_DIR

# Upload to TinyStore
aws s3 cp /tmp/$BACKUP_FILE s3://$BUCKET/ \
  --endpoint-url http://localhost:9000

# Cleanup
rm /tmp/$BACKUP_FILE

# Delete backups older than 30 days
aws s3 ls s3://$BUCKET/ --endpoint-url http://localhost:9000 | \
  while read -r line; do
    file_date=$(echo $line | awk '{print $1" "$2}')
    file_name=$(echo $line | awk '{print $4}')
    file_epoch=$(date -d "$file_date" +%s)
    current_epoch=$(date +%s)
    days_old=$(( ($current_epoch - $file_epoch) / 86400 ))

    if [ $days_old -gt 30 ]; then
      echo "Deleting old backup: $file_name"
      aws s3 rm s3://$BUCKET/$file_name --endpoint-url http://localhost:9000
    fi
  done

echo "Backup completed: $BACKUP_FILE"
```

### Image Upload Service (Python Flask)

```python
from flask import Flask, request, jsonify
import boto3
from werkzeug.utils import secure_filename

app = Flask(__name__)

s3 = boto3.client(
    's3',
    endpoint_url='http://localhost:9000',
    aws_access_key_id='tinystore',
    aws_secret_access_key='tinystore123',
    region_name='us-east-1'
)

BUCKET = 'images'

@app.route('/upload', methods=['POST'])
def upload():
    if 'file' not in request.files:
        return jsonify({'error': 'No file provided'}), 400

    file = request.files['file']
    if file.filename == '':
        return jsonify({'error': 'No file selected'}), 400

    filename = secure_filename(file.filename)

    try:
        s3.upload_fileobj(
            file,
            BUCKET,
            filename,
            ExtraArgs={'ContentType': file.content_type}
        )
        return jsonify({
            'message': 'File uploaded successfully',
            'filename': filename,
            'url': f'http://localhost:9000/{BUCKET}/{filename}'
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 500

if __name__ == '__main__':
    # Create bucket if not exists
    try:
        s3.create_bucket(Bucket=BUCKET)
    except:
        pass

    app.run(debug=True)
```

### CI/CD Artifact Storage (GitHub Actions)

```yaml
name: Build and Upload

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Build
        run: make build

      - name: Configure AWS CLI
        run: |
          aws configure set aws_access_key_id ${{ secrets.TINYSTORE_ACCESS_KEY }}
          aws configure set aws_secret_access_key ${{ secrets.TINYSTORE_SECRET_KEY }}
          aws configure set default.region us-east-1

      - name: Upload artifacts
        run: |
          aws s3 cp target/release/app \
            s3://artifacts/${{ github.sha }}/app \
            --endpoint-url ${{ secrets.TINYSTORE_ENDPOINT }}
```

## See Also

- [API Documentation](API.md)
- [Deployment Guide](DEPLOYMENT.md)
