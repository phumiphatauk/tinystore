//! S3 API error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tinystore_shared::StorageError;

/// S3 error response
#[derive(Debug, Serialize)]
#[serde(rename = "Error")]
pub struct S3ErrorResponse {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "RequestId")]
    pub request_id: String,
    #[serde(rename = "HostId")]
    pub host_id: String,
}

/// S3 error type
#[derive(Debug)]
pub struct S3Error {
    pub storage_error: StorageError,
    pub request_id: String,
}

impl S3Error {
    pub fn new(storage_error: StorageError) -> Self {
        Self {
            storage_error,
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn to_xml(&self) -> String {
        let response = S3ErrorResponse {
            code: self.storage_error.s3_code().to_string(),
            message: self.storage_error.to_string(),
            request_id: self.request_id.clone(),
            host_id: "tinystore".to_string(),
        };

        crate::xml::to_xml_string(&response)
            .unwrap_or_else(|_| "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Error><Code>InternalError</Code><Message>Failed to serialize error</Message></Error>".to_string())
    }
}

impl IntoResponse for S3Error {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.storage_error.status_code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let body = self.to_xml();

        (
            status,
            [("Content-Type", "application/xml")],
            body,
        ).into_response()
    }
}

impl From<StorageError> for S3Error {
    fn from(error: StorageError) -> Self {
        S3Error::new(error)
    }
}

pub type S3Result<T> = Result<T, S3Error>;
