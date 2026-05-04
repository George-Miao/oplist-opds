use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use snafu::Snafu;

pub type Result<T, E = AppError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum AppError {
    /// Failed to call the OpenList API for the given path (network / parse)
    #[snafu(display("OpenList request failed for path {path:?}: {source}"))]
    OpenListRequest { path: String, source: cyper::Error },

    /// OpenList returned a non-success response code inside its JSON envelope
    #[snafu(display("OpenList error {code} for path {path:?}: {message}"))]
    OpenListApi {
        path: String,
        code: i32,
        message: String,
    },

    /// The requested path was not found in OpenList
    #[snafu(display("Path not found: {path:?}"))]
    NotFound { path: String },

    /// Failed to serialise an OPDS feed to XML
    #[snafu(display("XML serialisation error: {source}"))]
    XmlSerialise { source: quick_xml::Error },

    /// Generic internal error (catch-all for unexpected conditions)
    #[snafu(display("Internal error: {message}"))]
    Internal { message: String },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::OpenListRequest { .. } => StatusCode::BAD_GATEWAY,
            AppError::OpenListApi { code, .. } if *code == 403 => StatusCode::FORBIDDEN,
            AppError::OpenListApi { .. } => StatusCode::BAD_GATEWAY,
            AppError::XmlSerialise { .. } | AppError::Internal { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        tracing::error!(error = %self, "request error");
        (status, self.to_string()).into_response()
    }
}
