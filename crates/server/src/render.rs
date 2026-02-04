use std::fmt::Debug;

use axum::{http::HeaderValue, response::IntoResponse};
use hyper::{StatusCode, header};
use serde::Serialize;

#[derive(Debug, serde::Serialize)]
pub struct Payload<T: serde::Serialize> {
    pub(crate) code: u16,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<T>,
}

impl<T: serde::Serialize> Default for Payload<T> {
    fn default() -> Self {
        Self {
            code: 200,
            message: Default::default(),
            data: Default::default(),
        }
    }
}

impl<T: serde::Serialize> Payload<T> {
    pub fn set_code(&mut self, code: u16) {
        self.code = code;
    }
    pub fn with_code(mut self, code: u16) -> Self {
        self.code = code;
        self
    }

    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }
    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = message.into();
        self
    }

    pub fn set_data(&mut self, data: T) {
        self.data = Some(data);
    }

    pub fn with_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }

    pub fn ok(data: T) -> Self {
        Self {
            code: 200,
            message: "".into(),
            data: Some(data),
        }
    }

    pub fn err<S: Into<String>>(error: S) -> Self {
        Self {
            code: 200,
            message: error.into(),
            data: Default::default(),
        }
    }
    pub fn bad_request<S: Into<String>>(error: S) -> Self {
        Self {
            code: 400,
            message: error.into(),
            data: Default::default(),
        }
    }

    // 没有权限
    pub fn no_permission<S: Into<String>>(error: S) -> Self {
        Self {
            code: 403,
            message: error.into(),
            data: Default::default(),
        }
    }
}

impl<T> IntoResponse for Payload<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        match serde_json::to_vec(&self) {
            Ok(bytes) => {
                let status_code = match self.code {
                    400 => StatusCode::BAD_REQUEST,
                    401 => StatusCode::UNAUTHORIZED,
                    403 => StatusCode::FORBIDDEN,
                    500 => StatusCode::INTERNAL_SERVER_ERROR,
                    200 => StatusCode::OK,
                    _ => StatusCode::OK,
                };
                (
                    status_code,
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("application/json"),
                    )],
                    bytes,
                )
                    .into_response()
            }
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}
