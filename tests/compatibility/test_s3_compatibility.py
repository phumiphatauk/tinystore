#!/usr/bin/env python3
"""
S3 Compatibility Tests using boto3

These tests verify that TinyStore is compatible with the AWS S3 SDK (boto3).
Run these tests against a running TinyStore server.

Prerequisites:
    pip install boto3 pytest

Usage:
    # Start TinyStore server first
    cargo run --release -- serve

    # Then run tests
    pytest tests/compatibility/test_s3_compatibility.py
"""

import boto3
import pytest
from botocore.client import Config
from botocore.exceptions import ClientError


# Configuration for TinyStore
ENDPOINT_URL = "http://localhost:9000"
ACCESS_KEY = "tinystore"
SECRET_KEY = "tinystore123"
REGION = "us-east-1"


@pytest.fixture(scope="session")
def s3_client():
    """Create a boto3 S3 client configured for TinyStore."""
    return boto3.client(
        "s3",
        endpoint_url=ENDPOINT_URL,
        aws_access_key_id=ACCESS_KEY,
        aws_secret_access_key=SECRET_KEY,
        region_name=REGION,
        config=Config(signature_version="s3v4"),
    )


@pytest.fixture(scope="function")
def test_bucket(s3_client):
    """Create a test bucket and clean it up after the test."""
    bucket_name = "test-bucket-compat"

    # Clean up if exists
    try:
        s3_client.delete_bucket(Bucket=bucket_name)
    except ClientError:
        pass

    # Create bucket
    s3_client.create_bucket(Bucket=bucket_name)

    yield bucket_name

    # Clean up
    try:
        # Delete all objects first
        response = s3_client.list_objects_v2(Bucket=bucket_name)
        if "Contents" in response:
            for obj in response["Contents"]:
                s3_client.delete_object(Bucket=bucket_name, Key=obj["Key"])

        # Delete bucket
        s3_client.delete_bucket(Bucket=bucket_name)
    except ClientError:
        pass


def test_list_buckets(s3_client):
    """Test listing buckets."""
    response = s3_client.list_buckets()
    assert "Buckets" in response
    assert isinstance(response["Buckets"], list)


def test_create_and_delete_bucket(s3_client):
    """Test creating and deleting a bucket."""
    bucket_name = "test-create-delete"

    # Create bucket
    response = s3_client.create_bucket(Bucket=bucket_name)
    assert response["ResponseMetadata"]["HTTPStatusCode"] == 200

    # Verify bucket exists
    response = s3_client.list_buckets()
    bucket_names = [b["Name"] for b in response["Buckets"]]
    assert bucket_name in bucket_names

    # Delete bucket
    response = s3_client.delete_bucket(Bucket=bucket_name)
    assert response["ResponseMetadata"]["HTTPStatusCode"] == 204


def test_head_bucket(s3_client, test_bucket):
    """Test HeadBucket operation."""
    response = s3_client.head_bucket(Bucket=test_bucket)
    assert response["ResponseMetadata"]["HTTPStatusCode"] == 200


def test_put_and_get_object(s3_client, test_bucket):
    """Test putting and getting an object."""
    key = "test-file.txt"
    content = b"Hello, TinyStore!"

    # Put object
    response = s3_client.put_object(Bucket=test_bucket, Key=key, Body=content)
    assert response["ResponseMetadata"]["HTTPStatusCode"] == 200
    assert "ETag" in response

    # Get object
    response = s3_client.get_object(Bucket=test_bucket, Key=key)
    assert response["ResponseMetadata"]["HTTPStatusCode"] == 200
    assert response["Body"].read() == content
    assert "ETag" in response


def test_head_object(s3_client, test_bucket):
    """Test HeadObject operation."""
    key = "test-head.txt"
    content = b"Test content"

    # Put object
    s3_client.put_object(Bucket=test_bucket, Key=key, Body=content)

    # Head object
    response = s3_client.head_object(Bucket=test_bucket, Key=key)
    assert response["ResponseMetadata"]["HTTPStatusCode"] == 200
    assert response["ContentLength"] == len(content)
    assert "ETag" in response


def test_delete_object(s3_client, test_bucket):
    """Test deleting an object."""
    key = "test-delete.txt"

    # Put object
    s3_client.put_object(Bucket=test_bucket, Key=key, Body=b"Delete me")

    # Delete object
    response = s3_client.delete_object(Bucket=test_bucket, Key=key)
    assert response["ResponseMetadata"]["HTTPStatusCode"] == 204

    # Verify object is gone
    with pytest.raises(ClientError) as exc:
        s3_client.head_object(Bucket=test_bucket, Key=key)
    assert exc.value.response["Error"]["Code"] == "404"


def test_list_objects_v2(s3_client, test_bucket):
    """Test ListObjectsV2 operation."""
    # Put multiple objects
    keys = ["file1.txt", "file2.txt", "dir/file3.txt"]
    for key in keys:
        s3_client.put_object(Bucket=test_bucket, Key=key, Body=b"data")

    # List all objects
    response = s3_client.list_objects_v2(Bucket=test_bucket)
    assert response["KeyCount"] == 3
    returned_keys = [obj["Key"] for obj in response["Contents"]]
    assert set(returned_keys) == set(keys)


def test_list_objects_with_prefix(s3_client, test_bucket):
    """Test ListObjectsV2 with prefix."""
    # Put objects with different prefixes
    s3_client.put_object(Bucket=test_bucket, Key="logs/2024/file1.txt", Body=b"data")
    s3_client.put_object(Bucket=test_bucket, Key="logs/2024/file2.txt", Body=b"data")
    s3_client.put_object(Bucket=test_bucket, Key="images/pic1.jpg", Body=b"data")

    # List with prefix
    response = s3_client.list_objects_v2(Bucket=test_bucket, Prefix="logs/")
    assert response["KeyCount"] == 2
    for obj in response["Contents"]:
        assert obj["Key"].startswith("logs/")


def test_list_objects_with_delimiter(s3_client, test_bucket):
    """Test ListObjectsV2 with delimiter."""
    # Create directory structure
    s3_client.put_object(Bucket=test_bucket, Key="root1.txt", Body=b"data")
    s3_client.put_object(Bucket=test_bucket, Key="dir1/file1.txt", Body=b"data")
    s3_client.put_object(Bucket=test_bucket, Key="dir2/file2.txt", Body=b"data")

    # List with delimiter
    response = s3_client.list_objects_v2(Bucket=test_bucket, Delimiter="/")

    # Should return root files and common prefixes
    assert response["KeyCount"] == 1  # root1.txt
    assert "CommonPrefixes" in response
    prefixes = [p["Prefix"] for p in response["CommonPrefixes"]]
    assert set(prefixes) == {"dir1/", "dir2/"}


def test_copy_object(s3_client, test_bucket):
    """Test copying an object."""
    source_key = "source.txt"
    dest_key = "destination.txt"
    content = b"Copy this content"

    # Put source object
    s3_client.put_object(Bucket=test_bucket, Key=source_key, Body=content)

    # Copy object
    copy_source = {"Bucket": test_bucket, "Key": source_key}
    response = s3_client.copy_object(
        Bucket=test_bucket, Key=dest_key, CopySource=copy_source
    )
    assert response["ResponseMetadata"]["HTTPStatusCode"] == 200

    # Verify destination object
    response = s3_client.get_object(Bucket=test_bucket, Key=dest_key)
    assert response["Body"].read() == content


def test_range_request(s3_client, test_bucket):
    """Test range requests (partial object retrieval)."""
    key = "range-test.txt"
    content = b"0123456789ABCDEFGHIJ"

    # Put object
    s3_client.put_object(Bucket=test_bucket, Key=key, Body=content)

    # Get range
    response = s3_client.get_object(Bucket=test_bucket, Key=key, Range="bytes=5-9")
    assert response["Body"].read() == b"56789"
    assert response["ContentRange"] == f"bytes 5-9/{len(content)}"


def test_multipart_upload(s3_client, test_bucket):
    """Test multipart upload."""
    key = "large-file.bin"

    # Create multipart upload
    response = s3_client.create_multipart_upload(Bucket=test_bucket, Key=key)
    upload_id = response["UploadId"]

    try:
        # Upload parts
        parts = []
        for i in range(1, 4):
            part_data = f"Part {i} data\n".encode()
            response = s3_client.upload_part(
                Bucket=test_bucket,
                Key=key,
                PartNumber=i,
                UploadId=upload_id,
                Body=part_data,
            )
            parts.append({"PartNumber": i, "ETag": response["ETag"]})

        # Complete multipart upload
        response = s3_client.complete_multipart_upload(
            Bucket=test_bucket,
            Key=key,
            UploadId=upload_id,
            MultipartUpload={"Parts": parts},
        )
        assert response["ResponseMetadata"]["HTTPStatusCode"] == 200

        # Verify object
        response = s3_client.get_object(Bucket=test_bucket, Key=key)
        content = response["Body"].read()
        assert b"Part 1 data" in content
        assert b"Part 2 data" in content
        assert b"Part 3 data" in content

    except Exception:
        # Abort on failure
        s3_client.abort_multipart_upload(
            Bucket=test_bucket, Key=key, UploadId=upload_id
        )
        raise


def test_abort_multipart_upload(s3_client, test_bucket):
    """Test aborting a multipart upload."""
    key = "aborted-file.bin"

    # Create multipart upload
    response = s3_client.create_multipart_upload(Bucket=test_bucket, Key=key)
    upload_id = response["UploadId"]

    # Upload a part
    s3_client.upload_part(
        Bucket=test_bucket,
        Key=key,
        PartNumber=1,
        UploadId=upload_id,
        Body=b"Part data",
    )

    # Abort upload
    response = s3_client.abort_multipart_upload(
        Bucket=test_bucket, Key=key, UploadId=upload_id
    )
    assert response["ResponseMetadata"]["HTTPStatusCode"] == 204

    # Verify object was not created
    with pytest.raises(ClientError) as exc:
        s3_client.head_object(Bucket=test_bucket, Key=key)
    assert exc.value.response["Error"]["Code"] == "404"


def test_object_not_found(s3_client, test_bucket):
    """Test accessing non-existent object."""
    with pytest.raises(ClientError) as exc:
        s3_client.get_object(Bucket=test_bucket, Key="nonexistent.txt")
    assert exc.value.response["Error"]["Code"] == "NoSuchKey"


def test_bucket_not_found(s3_client):
    """Test accessing non-existent bucket."""
    with pytest.raises(ClientError) as exc:
        s3_client.head_bucket(Bucket="nonexistent-bucket-12345")
    assert exc.value.response["Error"]["Code"] in ["404", "NoSuchBucket"]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
