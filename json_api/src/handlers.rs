use parking_lot::Mutex;

use client::client_proxy::ClientProxy;
use rocket::State;
use rocket_contrib::json::Json;

use crate::error::Result;

pub struct AppState {
    // TODO: A pool of client proxies can be used instead of a mutex if needed.
    pub proxy: ClientProxy,
}

#[derive(Serialize)]
pub struct Balance {
    balance: String,
}

#[get("/get_balance/<addr>")]
pub fn get_balance(state: State<Mutex<AppState>>, addr: String) -> Result<Json<Balance>> {
    // It seems like the first element of the space_delim_strings argument is not used.
    let balance = state.lock().proxy.get_balance(&vec!["", &addr])?;

    Ok(Json(Balance { balance }))
}
