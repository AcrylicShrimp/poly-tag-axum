use axum::{
    extract::multipart::{MultipartError, MultipartRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileRouterError {
    #[error("invalid multipart request")]
    MultipartExtractorRejection(#[from] MultipartRejection),
    #[error("invalid multipart request")]
    MultipartError(#[from] MultipartError),
    #[error("internal server error")]
    IOError(#[from] std::io::Error),
}

impl IntoResponse for FileRouterError {
    fn into_response(self) -> Response {
        #[cfg(debug_assertions)]
        let body = Json(json!({ "error": format!("{:?}", self) }));
        #[cfg(not(debug_assertions))]
        let body = Json(json!({
            "error": self.to_string()
        }));
        let status_code = match self {
            FileRouterError::MultipartExtractorRejection(_) => StatusCode::BAD_REQUEST,
            // FileRouterError::MultipartError(err) => {
            //     status_code_from_multer_error(err.into_multer())
            // }
            FileRouterError::MultipartError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FileRouterError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status_code, body).into_response()
    }
}

// fn status_code_from_multer_error(err: axam::multer::Error) -> StatusCode {
//     match err {
//         axam::multer::Error::UnknownField { .. } => StatusCode::BAD_REQUEST,
//         axam::multer::Error::IncompleteFieldData { .. } => StatusCode::BAD_REQUEST,
//         axam::multer::Error::IncompleteHeaders => StatusCode::BAD_REQUEST,
//         axam::multer::Error::ReadHeaderFailed(..) => StatusCode::BAD_REQUEST,
//         axam::multer::Error::DecodeHeaderName { .. } => StatusCode::BAD_REQUEST,
//         axam::multer::Error::DecodeHeaderValue { .. } => StatusCode::BAD_REQUEST,
//         axam::multer::Error::IncompleteStream => StatusCode::BAD_REQUEST,
//         axam::multer::Error::FieldSizeExceeded { .. } => StatusCode::PAYLOAD_TOO_LARGE,
//         axam::multer::Error::StreamSizeExceeded { .. } => StatusCode::PAYLOAD_TOO_LARGE,
//         axam::multer::Error::StreamReadFailed(err) => {
//             match err.downcast_ref::<axam::multer::Error>() {
//                 Some(_) => {
//                     let err = *err.downcast::<axam::multer::Error>().unwrap();
//                     return status_code_from_multer_error(err);
//                 }
//                 None => {}
//             }

//             match err.downcast_ref::<axam::Error>() {
//                 Some(_) => {
//                     let err = *err.downcast::<axam::Error>().unwrap();
//                     return status_code_from_axum_error(err);
//                 }
//                 None => {}
//             }

//             StatusCode::INTERNAL_SERVER_ERROR
//         }
//         axam::multer::Error::LockFailure => StatusCode::INTERNAL_SERVER_ERROR,
//         axam::multer::Error::NoMultipart => StatusCode::BAD_REQUEST,
//         axam::multer::Error::DecodeContentType(..) => StatusCode::BAD_REQUEST,
//         axam::multer::Error::NoBoundary => StatusCode::BAD_REQUEST,
//         _ => StatusCode::BAD_REQUEST,
//     }
// }

// fn status_code_from_axum_error(err: axam::Error) -> StatusCode {
//     let err = err.into_inner();

//     match err.downcast_ref::<axam::extract::rejection::LengthLimitError>() {
//         Some(_) => return StatusCode::PAYLOAD_TOO_LARGE,
//         None => {}
//     }

//     match err.downcast_ref::<http_body::LengthLimitError>() {
//         Some(_) => return StatusCode::PAYLOAD_TOO_LARGE,
//         None => {}
//     }

//     match err.downcast_ref::<axam::Error>() {
//         Some(_) => {
//             let err = *err.downcast::<axam::Error>().unwrap();
//             return status_code_from_axum_error(err);
//         }
//         None => {}
//     }

//     StatusCode::INTERNAL_SERVER_ERROR
// }
