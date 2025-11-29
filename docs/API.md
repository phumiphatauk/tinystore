# TinyStore API Documentation

TinyStore implements a subset of the Amazon S3 API, making it compatible with most S3 clients and SDKs.

## Base URL

```
http://localhost:9000
```

## Authentication

TinyStore uses AWS Signature Version 4 authentication. Configure your S3 client with:

- **Access Key**: Your configured access key (e.g., `tinystore`)
- **Secret Key**: Your configured secret key (e.g., `tinystore123`)
- **Region**: `us-east-1` (or as configured)

## Supported Operations

### Bucket Operations

#### Create Bucket

```http
PUT /{bucket} HTTP/1.1
Host: localhost:9000
```

**Example (AWS CLI):**
```bash
aws s3 mb s3://my-bucket --endpoint-url http://localhost:9000
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<CreateBucketConfiguration>
  <Location>us-east-1</Location>
</CreateBucketConfiguration>
```

#### List Buckets

```http
GET / HTTP/1.1
Host: localhost:9000
```

**Example (AWS CLI):**
```bash
aws s3 ls --endpoint-url http://localhost:9000
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult>
  <Owner>
    <ID>tinystore</ID>
    <DisplayName>tinystore</DisplayName>
  </Owner>
  <Buckets>
    <Bucket>
      <Name>my-bucket</Name>
      <CreationDate>2024-01-01T00:00:00.000Z</CreationDate>
    </Bucket>
  </Buckets>
</ListAllMyBucketsResult>
```

#### Head Bucket

```http
HEAD /{bucket} HTTP/1.1
Host: localhost:9000
```

**Example (AWS CLI):**
```bash
aws s3api head-bucket --bucket my-bucket --endpoint-url http://localhost:9000
```

#### Delete Bucket

```http
DELETE /{bucket} HTTP/1.1
Host: localhost:9000
```

**Example (AWS CLI):**
```bash
aws s3 rb s3://my-bucket --endpoint-url http://localhost:9000
```

### Object Operations

#### Put Object

```http
PUT /{bucket}/{key} HTTP/1.1
Host: localhost:9000
Content-Length: {size}
Content-Type: {mime-type}

{object-data}
```

**Example (AWS CLI):**
```bash
aws s3 cp file.txt s3://my-bucket/file.txt --endpoint-url http://localhost:9000
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<PutObjectResult>
  <ETag>"abc123..."</ETag>
</PutObjectResult>
```

#### Get Object

```http
GET /{bucket}/{key} HTTP/1.1
Host: localhost:9000
```

**Example (AWS CLI):**
```bash
aws s3 cp s3://my-bucket/file.txt file.txt --endpoint-url http://localhost:9000
```

**Response Headers:**
- `Content-Type`: Object MIME type
- `Content-Length`: Object size
- `ETag`: Object ETag
- `Last-Modified`: Last modification date

#### Head Object

```http
HEAD /{bucket}/{key} HTTP/1.1
Host: localhost:9000
```

**Example (AWS CLI):**
```bash
aws s3api head-object --bucket my-bucket --key file.txt --endpoint-url http://localhost:9000
```

**Response Headers:**
- `Content-Type`: Object MIME type
- `Content-Length`: Object size
- `ETag`: Object ETag
- `Last-Modified`: Last modification date

#### Delete Object

```http
DELETE /{bucket}/{key} HTTP/1.1
Host: localhost:9000
```

**Example (AWS CLI):**
```bash
aws s3 rm s3://my-bucket/file.txt --endpoint-url http://localhost:9000
```

#### List Objects (v2)

```http
GET /{bucket}?list-type=2&prefix={prefix}&max-keys={max} HTTP/1.1
Host: localhost:9000
```

**Query Parameters:**
- `list-type=2`: Use v2 API
- `prefix`: Filter by prefix (optional)
- `max-keys`: Maximum number of keys to return (default: 1000)
- `continuation-token`: Token for pagination (optional)

**Example (AWS CLI):**
```bash
aws s3 ls s3://my-bucket/ --endpoint-url http://localhost:9000
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
  <Name>my-bucket</Name>
  <Prefix></Prefix>
  <KeyCount>2</KeyCount>
  <MaxKeys>1000</MaxKeys>
  <IsTruncated>false</IsTruncated>
  <Contents>
    <Key>file1.txt</Key>
    <LastModified>2024-01-01T00:00:00.000Z</LastModified>
    <ETag>"abc123..."</ETag>
    <Size>1024</Size>
    <StorageClass>STANDARD</StorageClass>
  </Contents>
  <Contents>
    <Key>file2.txt</Key>
    <LastModified>2024-01-01T00:00:00.000Z</LastModified>
    <ETag>"def456..."</ETag>
    <Size>2048</Size>
    <StorageClass>STANDARD</StorageClass>
  </Contents>
</ListBucketResult>
```

#### Copy Object

```http
PUT /{dest-bucket}/{dest-key} HTTP/1.1
Host: localhost:9000
x-amz-copy-source: /{source-bucket}/{source-key}
```

**Example (AWS CLI):**
```bash
aws s3 cp s3://my-bucket/file.txt s3://my-bucket/copy.txt --endpoint-url http://localhost:9000
```

### Multipart Upload

#### Initiate Multipart Upload

```http
POST /{bucket}/{key}?uploads HTTP/1.1
Host: localhost:9000
```

**Example (AWS CLI):**
```bash
aws s3api create-multipart-upload --bucket my-bucket --key large-file.bin --endpoint-url http://localhost:9000
```

#### Upload Part

```http
PUT /{bucket}/{key}?partNumber={num}&uploadId={id} HTTP/1.1
Host: localhost:9000
Content-Length: {size}

{part-data}
```

#### Complete Multipart Upload

```http
POST /{bucket}/{key}?uploadId={id} HTTP/1.1
Host: localhost:9000

<CompleteMultipartUpload>
  <Part>
    <PartNumber>1</PartNumber>
    <ETag>"etag1"</ETag>
  </Part>
  <Part>
    <PartNumber>2</PartNumber>
    <ETag>"etag2"</ETag>
  </Part>
</CompleteMultipartUpload>
```

#### Abort Multipart Upload

```http
DELETE /{bucket}/{key}?uploadId={id} HTTP/1.1
Host: localhost:9000
```

### Bulk Operations

#### Delete Multiple Objects

```http
POST /{bucket}?delete HTTP/1.1
Host: localhost:9000
Content-MD5: {md5}

<Delete>
  <Object>
    <Key>key1</Key>
  </Object>
  <Object>
    <Key>key2</Key>
  </Object>
</Delete>
```

**Example (AWS CLI):**
```bash
aws s3 rm s3://my-bucket --recursive --endpoint-url http://localhost:9000
```

## Error Responses

All errors follow the S3 XML error format:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Error>
  <Code>NoSuchBucket</Code>
  <Message>The specified bucket does not exist</Message>
  <Resource>/my-bucket</Resource>
  <RequestId>abc123</RequestId>
</Error>
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `NoSuchBucket` | 404 | Bucket does not exist |
| `BucketAlreadyExists` | 409 | Bucket already exists |
| `NoSuchKey` | 404 | Object does not exist |
| `InvalidAccessKeyId` | 403 | Invalid access key |
| `SignatureDoesNotMatch` | 403 | Invalid signature |
| `AccessDenied` | 403 | Permission denied |
| `InvalidBucketName` | 400 | Invalid bucket name |
| `EntityTooLarge` | 400 | Object too large |
| `BucketNotEmpty` | 409 | Cannot delete non-empty bucket |

## Health Check

TinyStore provides a health check endpoint:

```http
GET /health HTTP/1.1
Host: localhost:9000
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

## Metrics (Optional)

If metrics are enabled, they're available at:

```http
GET /metrics HTTP/1.1
Host: localhost:9000
```

## Web UI

The web-based management UI is available at:

```
http://localhost:9000/ui
```

Features:
- Browse buckets and objects
- Upload/download files
- Delete objects and buckets
- View object metadata

## Limitations

### Current Limitations

1. **Object Size**: Maximum object size is 5GB by default (configurable)
2. **Multipart Parts**: Maximum 10,000 parts per upload
3. **Keys per List**: Maximum 1,000 keys per list operation
4. **Bucket Count**: Maximum 100 buckets (configurable)

### Unsupported Features

The following S3 features are not yet supported:

- Object versioning
- Object lifecycle policies
- Bucket policies
- Access control lists (ACLs)
- Server-side encryption
- Cross-region replication
- Event notifications
- Object tagging
- Object locking

## SDK Examples

### Python (boto3)

```python
import boto3

s3 = boto3.client(
    's3',
    endpoint_url='http://localhost:9000',
    aws_access_key_id='tinystore',
    aws_secret_access_key='tinystore123',
    region_name='us-east-1'
)

# Create bucket
s3.create_bucket(Bucket='my-bucket')

# Upload file
s3.upload_file('file.txt', 'my-bucket', 'file.txt')

# Download file
s3.download_file('my-bucket', 'file.txt', 'downloaded.txt')
```

### JavaScript (AWS SDK v3)

```javascript
const { S3Client, PutObjectCommand } = require('@aws-sdk/client-s3');

const client = new S3Client({
  endpoint: 'http://localhost:9000',
  region: 'us-east-1',
  credentials: {
    accessKeyId: 'tinystore',
    secretAccessKey: 'tinystore123',
  },
  forcePathStyle: true,
});

// Upload object
await client.send(new PutObjectCommand({
  Bucket: 'my-bucket',
  Key: 'file.txt',
  Body: 'Hello, World!',
}));
```

### Go (AWS SDK)

```go
package main

import (
    "github.com/aws/aws-sdk-go/aws"
    "github.com/aws/aws-sdk-go/aws/credentials"
    "github.com/aws/aws-sdk-go/aws/session"
    "github.com/aws/aws-sdk-go/service/s3"
)

func main() {
    sess := session.Must(session.NewSession(&aws.Config{
        Endpoint:         aws.String("http://localhost:9000"),
        Region:           aws.String("us-east-1"),
        Credentials:      credentials.NewStaticCredentials("tinystore", "tinystore123", ""),
        S3ForcePathStyle: aws.Bool(true),
    }))

    svc := s3.New(sess)

    // Create bucket
    svc.CreateBucket(&s3.CreateBucketInput{
        Bucket: aws.String("my-bucket"),
    })
}
```

## See Also

- [Deployment Guide](DEPLOYMENT.md)
- [Usage Examples](EXAMPLES.md)
- [AWS S3 API Reference](https://docs.aws.amazon.com/AmazonS3/latest/API/Welcome.html)
