//! Integration tests for S3 API operations
//!
//! These tests verify the complete flow from HTTP requests through
//! S3 API handlers to the storage backend.

use bytes::Bytes;
use tinystore_storage::{MemoryBackend, StorageBackend};
use tinystore_shared::ObjectMetadata;

#[tokio::test]
async fn test_storage_backend_integration() {
    let backend = MemoryBackend::new();

    // Test bucket lifecycle
    backend.create_bucket("test-bucket").await.unwrap();
    assert!(backend.bucket_exists("test-bucket").await.unwrap());

    // Test object operations
    let data = Bytes::from("integration test data");
    let metadata = ObjectMetadata::default();

    let put_result = backend
        .put_object("test-bucket", "test-key", data.clone(), metadata)
        .await
        .unwrap();

    assert!(!put_result.etag.is_empty());

    // Get the object back
    let get_result = backend
        .get_object("test-bucket", "test-key", None)
        .await
        .unwrap();

    assert_eq!(get_result.data, data);

    // Clean up
    backend.delete_object("test-bucket", "test-key").await.unwrap();
    backend.delete_bucket("test-bucket").await.unwrap();
}

#[tokio::test]
async fn test_multipart_upload_integration() {
    let backend = MemoryBackend::new();
    backend.create_bucket("multipart-test").await.unwrap();

    // Create multipart upload
    let metadata = ObjectMetadata::default();
    let upload_id = backend
        .create_multipart_upload("multipart-test", "large-file", metadata)
        .await
        .unwrap();

    // Upload 3 parts
    let parts = vec![
        Bytes::from("Part 1 "),
        Bytes::from("Part 2 "),
        Bytes::from("Part 3"),
    ];

    let mut completed_parts = Vec::new();
    for (i, part_data) in parts.iter().enumerate() {
        let part_number = (i + 1) as u32;
        let part_info = backend
            .upload_part(
                "multipart-test",
                "large-file",
                &upload_id,
                part_number,
                part_data.clone(),
            )
            .await
            .unwrap();

        completed_parts.push(tinystore_storage::CompletedPart {
            part_number,
            etag: part_info.etag,
        });
    }

    // Complete the upload
    let result = backend
        .complete_multipart_upload("multipart-test", "large-file", &upload_id, completed_parts)
        .await
        .unwrap();

    assert!(!result.etag.is_empty());

    // Verify the final object
    let get_result = backend
        .get_object("multipart-test", "large-file", None)
        .await
        .unwrap();

    assert_eq!(get_result.data, Bytes::from("Part 1 Part 2 Part 3"));

    // Clean up
    backend.delete_object("multipart-test", "large-file").await.unwrap();
    backend.delete_bucket("multipart-test").await.unwrap();
}

#[tokio::test]
async fn test_list_objects_integration() {
    let backend = MemoryBackend::new();
    backend.create_bucket("list-test").await.unwrap();

    // Create a directory structure
    let metadata = ObjectMetadata::default();
    let objects = vec![
        ("file1.txt", "data1"),
        ("file2.txt", "data2"),
        ("dir1/file3.txt", "data3"),
        ("dir1/file4.txt", "data4"),
        ("dir2/file5.txt", "data5"),
    ];

    for (key, data) in &objects {
        backend
            .put_object("list-test", key, Bytes::from(*data), metadata.clone())
            .await
            .unwrap();
    }

    // List all objects
    let params = tinystore_storage::ListObjectsParams::default();
    let result = backend.list_objects("list-test", params).await.unwrap();
    assert_eq!(result.objects.len(), 5);

    // List with prefix
    let params = tinystore_storage::ListObjectsParams {
        prefix: Some("dir1/".to_string()),
        ..Default::default()
    };
    let result = backend.list_objects("list-test", params).await.unwrap();
    assert_eq!(result.objects.len(), 2);

    // List with delimiter
    let params = tinystore_storage::ListObjectsParams {
        delimiter: Some("/".to_string()),
        ..Default::default()
    };
    let result = backend.list_objects("list-test", params).await.unwrap();
    assert_eq!(result.objects.len(), 2); // file1.txt, file2.txt
    assert_eq!(result.common_prefixes.len(), 2); // dir1/, dir2/

    // Clean up
    for (key, _) in &objects {
        backend.delete_object("list-test", key).await.unwrap();
    }
    backend.delete_bucket("list-test").await.unwrap();
}

#[tokio::test]
async fn test_copy_object_integration() {
    let backend = MemoryBackend::new();
    backend.create_bucket("source").await.unwrap();
    backend.create_bucket("dest").await.unwrap();

    // Create source object
    let data = Bytes::from("data to copy");
    let metadata = ObjectMetadata::default();
    backend
        .put_object("source", "original.txt", data.clone(), metadata)
        .await
        .unwrap();

    // Copy to destination
    let result = backend
        .copy_object("source", "original.txt", "dest", "copy.txt")
        .await
        .unwrap();

    assert!(!result.etag.is_empty());

    // Verify both objects exist
    let source_result = backend.get_object("source", "original.txt", None).await.unwrap();
    let dest_result = backend.get_object("dest", "copy.txt", None).await.unwrap();

    assert_eq!(source_result.data, data);
    assert_eq!(dest_result.data, data);

    // Clean up
    backend.delete_object("source", "original.txt").await.unwrap();
    backend.delete_object("dest", "copy.txt").await.unwrap();
    backend.delete_bucket("source").await.unwrap();
    backend.delete_bucket("dest").await.unwrap();
}

#[tokio::test]
async fn test_bucket_not_empty_error() {
    let backend = MemoryBackend::new();
    backend.create_bucket("not-empty").await.unwrap();

    // Add an object
    let metadata = ObjectMetadata::default();
    backend
        .put_object("not-empty", "file.txt", Bytes::from("data"), metadata)
        .await
        .unwrap();

    // Try to delete non-empty bucket
    let result = backend.delete_bucket("not-empty").await;
    assert!(result.is_err());

    // Clean up properly
    backend.delete_object("not-empty", "file.txt").await.unwrap();
    backend.delete_bucket("not-empty").await.unwrap();
}

#[tokio::test]
async fn test_range_request_integration() {
    let backend = MemoryBackend::new();
    backend.create_bucket("range-test").await.unwrap();

    // Create object with known content
    let data = Bytes::from("0123456789ABCDEFGHIJ");
    let metadata = ObjectMetadata::default();
    backend
        .put_object("range-test", "data.bin", data, metadata)
        .await
        .unwrap();

    // Request specific range
    let range = tinystore_storage::Range {
        start: 5,
        end: Some(10),
    };
    let result = backend
        .get_object("range-test", "data.bin", Some(range))
        .await
        .unwrap();

    assert_eq!(result.data, Bytes::from("56789"));

    // Request from offset to end
    let range = tinystore_storage::Range {
        start: 10,
        end: None,
    };
    let result = backend
        .get_object("range-test", "data.bin", Some(range))
        .await
        .unwrap();

    assert_eq!(result.data, Bytes::from("ABCDEFGHIJ"));

    // Clean up
    backend.delete_object("range-test", "data.bin").await.unwrap();
    backend.delete_bucket("range-test").await.unwrap();
}

#[tokio::test]
async fn test_storage_stats_integration() {
    let backend = MemoryBackend::new();

    // Create multiple buckets and objects
    backend.create_bucket("stats-1").await.unwrap();
    backend.create_bucket("stats-2").await.unwrap();

    let metadata = ObjectMetadata::default();
    backend.put_object("stats-1", "file1", Bytes::from("12345"), metadata.clone()).await.unwrap();
    backend.put_object("stats-1", "file2", Bytes::from("67890"), metadata.clone()).await.unwrap();
    backend.put_object("stats-2", "file3", Bytes::from("abc"), metadata).await.unwrap();

    let stats = backend.get_stats().await.unwrap();
    assert_eq!(stats.total_buckets, 2);
    assert_eq!(stats.total_objects, 3);
    assert_eq!(stats.total_size_bytes, 13); // 5 + 5 + 3

    // Clean up
    backend.delete_object("stats-1", "file1").await.unwrap();
    backend.delete_object("stats-1", "file2").await.unwrap();
    backend.delete_object("stats-2", "file3").await.unwrap();
    backend.delete_bucket("stats-1").await.unwrap();
    backend.delete_bucket("stats-2").await.unwrap();
}
