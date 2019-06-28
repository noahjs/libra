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
    let mut proxy = &mut state.lock().proxy;
    let balance = proxy.get_balance_alt(&addr)?;
    
    Ok(Json(BalanceRes { balance }))
}

#[derive(FromForm)]
pub struct MintCoinsData {
    receiver: String,
    num_coins: String,
}

#[post("/mint_coins", data = "<data>")]
pub fn mint_coins(state: State<Mutex<AppState>>, data: Form<MintCoinsData>) -> Result<Json<JsonValue>> {
    let mut proxy = &mut state.lock().proxy;
    proxy.mint_coins_alt(&data.receiver, &data.num_coins)?;
    
    Ok(Json(json!({ "success": true })))
}

#[derive(FromForm)]
pub struct TransferCoinsData {
    sender_addr: String,
    receiver_addr: String,
    num_coins: String,
    gas_unit_price: Option<u64>,
    max_gas_amount: Option<u64>,
}

#[post("/transfer_coins", data = "<data>")]
pub fn transfer_coins(state: State<Mutex<AppState>>, data: Form<TransferCoinsData>) -> Result<Json<JsonValue>> {
    let mut proxy = &mut state.lock().proxy;
    proxy.transfer_coins_alt(&data.sender_addr, &data.receiver_addr, &data.num_coins, data.gas_unit_price, data.max_gas_amount)?;

    Ok(Json(json!({ "success": true })))
}
