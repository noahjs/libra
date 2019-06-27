use parking_lot::Mutex;

use client::client_proxy::ClientProxy;
use rocket::State;
use rocket::request::Form;
use rocket_contrib::json::Json;
use serde_json::{json, Value as JsonValue};

use crate::error::Result;

pub struct AppState {
    // TODO: A pool of client proxies can be used instead of a mutex if needed.
    pub proxy: ClientProxy,
}

#[derive(Serialize)]
pub struct BalanceRes {
    balance: String,
}

#[get("/get_balance/<addr>")]
pub fn get_balance(state: State<Mutex<AppState>>, addr: String) -> Result<Json<BalanceRes>> {
    // It seems like the first element of the space_delim_strings argument is not used.
    let balance = state.lock().proxy.get_balance(&["", &addr])?;

    Ok(Json(BalanceRes { balance }))
}

#[derive(FromForm)]
pub struct MintCoinsData {
    receiver: String,
    num_coins: String,
}

#[post("/mint_coins", data = "<data>")]
pub fn mint_coins(state: State<Mutex<AppState>>, data: Form<MintCoinsData>) -> Result<Json<JsonValue>> {
    // TODO: Should it be blocking?
    state.lock().proxy.mint_coins(&["", &data.receiver, &data.num_coins], true)?;
    Ok(Json(json!({ "success": true })))
}

