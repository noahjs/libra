use failure::Error as FailureError;
use libra_wallet::error::WalletError;
use rocket_contrib::json::Json;

// TODO: Errors can be detailed further if needed.
pub type Result<T> = ::std::result::Result<T, ApiError>;

#[derive(Debug, Responder)]
#[response(status = 500)]
pub struct ApiError(Json<ErrResponse>);

impl ApiError {
    pub fn new(error: String) -> Self {
        ApiError(Json(ErrResponse { error }))
    }
}

impl From<FailureError> for ApiError {
    fn from(err: FailureError) -> Self {
        ApiError::new(format!("{}", err))
    }
}

impl From<WalletError> for ApiError {
    fn from(err: WalletError) -> Self {
        ApiError::new(format!("Wallet error: {}", err))
    }
}

#[derive(Debug, Serialize)]
pub struct ErrResponse {
    error: String,
}
