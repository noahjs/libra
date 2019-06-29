use parking_lot::Mutex;
use rocket::{request::Form, State};
use rocket_contrib::json::Json;
use serde_json::{json, Value as JsonValue};

use client::client_proxy::ClientProxy;

use crate::{error::Result, serializers::*};

// TODO: Refactor to support multiple clients.

pub struct AppState {
    pub proxy: ClientProxy,
}

#[post("/create_next_account")]
pub fn create_next_account(state: State<Mutex<AppState>>) -> Result<Json<JsonValue>> {
    let proxy = &mut state.lock().proxy;

    let acc = proxy.create_next_account()?;
    
    Ok(Json(json!({
        "address": acc.address,
        "index": acc.index,
        "success": true,
    })))
}

#[get("/get_latest_account_state/<addr>")]
pub fn get_latest_account_state(state: State<Mutex<AppState>>, addr: String) -> Result<Json<AccountResourceSer>> {
    let proxy = &mut state.lock().proxy;
    let state = proxy.get_latest_account_resource(&addr)?;
    
    Ok(Json(state.into()))
}

#[derive(FromForm)]
pub struct MintCoinsData {
    receiver: String,
    num_coins: String,
}

#[post("/mint_coins", data = "<data>")]
pub fn mint_coins(
    state: State<Mutex<AppState>>,
    data: Form<MintCoinsData>,
) -> Result<Json<JsonValue>> {
    let proxy = &mut state.lock().proxy;

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
pub fn transfer_coins(
    state: State<Mutex<AppState>>,
    data: Form<TransferCoinsData>,
) -> Result<Json<JsonValue>> {
    let proxy = &mut state.lock().proxy;

    let result = proxy.transfer_coins_alt(
        &data.sender_addr,
        &data.receiver_addr,
        &data.num_coins,
        data.gas_unit_price,
        data.max_gas_amount,
    )?;

    Ok(Json(json!({
        "success": true,
        "index": result.account_index,
        "sequence": result.sequence_number,
    })))
}

#[get("/get_committed_txn_by_acc_seq/<addr>?<sequence_number>&<fetch_events>")]
pub fn get_committed_txn_by_acc_seq(
    state: State<Mutex<AppState>>,
    addr: String,
    sequence_number: u64,
    fetch_events: bool,
) -> Result<Json<Vec<TxWithEvents>>> {
    let proxy = &mut state.lock().proxy;

    proxy
        .get_committed_txn_by_acc_seq_alt(&addr, sequence_number, fetch_events)
        .map(|val| {
            let transactions = val
                .into_iter()
                .map(|(tx, events)| TxWithEvents {
                    transaction: tx,
                    events,
                })
                .collect();

            Json(transactions)
        })
        .map_err(|err| From::from(err))
}

#[get("/get_committed_txn_by_range?<start_version>&<limit>&<fetch_events>")]
pub fn get_committed_txn_by_range(
    state: State<Mutex<AppState>>,
    start_version: u64,
    limit: u64,
    fetch_events: bool,
) -> Result<Json<Vec<TxWithEvents>>> {
    let proxy = &mut state.lock().proxy;

    proxy
        .client
        .get_txn_by_range(start_version, limit, fetch_events)
        .map(|val| {
            let transactions = val
                .into_iter()
                .map(|(tx, events)| TxWithEvents {
                    transaction: tx,
                    events,
                })
                .collect();

            Json(transactions)
        })
        .map_err(|err| From::from(err))
}

#[get("/get_events_by_account_and_type/<addr>?<event_type>&<start_seq_number>&<limit>&<ascending>")]
pub fn get_events_by_account_and_type(
    state: State<Mutex<AppState>>,
    addr: String,
    event_type: String,
    start_seq_number: u64,
    limit: u64,
    ascending: bool,
) -> Result<Json<AccWithEvents>> {
    let proxy = &mut state.lock().proxy;

    proxy
        .get_events_by_account_and_type_alt(&addr, &event_type, start_seq_number, limit, ascending) // Don't fetch events: they are not serializable.
        .map(|(events, account)| Json(AccWithEvents { account, events }))
        .map_err(|err| From::from(err))
}

//#[derive(FromForm)]
//pub struct TxReqData {
//    program: ProgramKind, // TODO: Tag internally
//    sender: String,
//    gas_unit_price: Option<u64>,
//    max_gas_amount: Option<u64>,
//}
//
//#[derive(Deserialize)]
//#[serde(tag = "type", rename_all = "snake_case")]
//pub enum ProgramKind {
//    Transfer {
//        recipient: String,
//        amount: String,
//    },
//    Mint {
//        sender: String,
//        amount: String,
//    },
//}
//
//#[post("/create_submit_transaction_req", data = "<data>")]
//pub fn create_submit_transaction_req(state: State<Mutex<AppState>>, data: Form<TxReqData>) -> Result<Json<JsonValue>> {
//    let proxy = &mut state.lock().proxy;
//
//    let program = match data.program {
//        ProgramKind::Transfer { recipient, amount } => {
//            let recipient =
//                proxy.get_account_address_from_parameter(&recipient)?;
//            let amount = ClientProxy::convert_to_micro_libras_alt(&amount)?;
//            
//            vm_genesis::encode_transfer_program(&recipient, amount)
//        }
//        ProgramKind::Mint { sender, amount } => {
//            let sender =
//                proxy.get_account_address_from_parameter(&sender)?;
//            let amount = ClientProxy::convert_to_micro_libras_alt(&amount)?;
//            
//            vm_genesis::encode_mint_program(&sender, amount)
//        }
//    };
//    
//    let sender_address =
//        proxy.get_account_address_from_parameter(&data.sender)?;
//    let sender_ref_id = proxy.ref_id_by_address(&sender_address)?;
//    
//    // FIXME: Get account by address
////    let sender_account = proxy
//
//    let tx = proxy.create_submit_transaction_req(
//        program,
//        &data.sender_account,
//        data.gas_unit_price,
//        data.max_gas_amount,
//    )?;
//    
//
//    let sender_mut = proxy.account_mut(sender_ref_id)?;
//
//    resp = proxy.client.submit_transaction(sender_mut, &req)?;
//
//    Ok(Json(json!({ "success": true })))
//}
