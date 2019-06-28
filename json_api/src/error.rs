use failure::Error as FailureError;
use rocket_contrib::json::Json;

// TODO: Errors can be detailed further if needed.
pub type Result<T> = ::std::result::Result<T, ApiError>;

#[derive(Debug, Responder)]
#[response(status = 500)]
pub struct ApiError(Json<ErrResponse>);

#[derive(Debug, Serialize)]
pub struct ErrResponse {
    msg: String,
}

impl From<FailureError> for ApiError {
    fn from(err: FailureError) -> Self {
        ApiError(Json(ErrResponse {
            msg: format!("{}", err),
        }))
    }
}
