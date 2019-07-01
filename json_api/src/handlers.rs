use rocket::{request::Form, State};
use rocket_contrib::json::Json;
use serde_json::{json, Value as JsonValue};

use libra_wallet::{key_factory::ChildNumber, Mnemonic, WalletLibrary};
use types::{
    access_path::AccessPath,
    account_config::{account_received_event_path, account_sent_event_path},
};

use crate::{client::Client, error::Result, serializers::*, state::AppState, utils};

#[post("/create_wallet")]
pub fn create_wallet() -> Result<Json<JsonValue>> {
    let wallet = WalletLibrary::new();
    let mnemonic = wallet.mnemonic();

    Ok(Json(json!({ "mnemonic": mnemonic })))
}

#[derive(FromForm)]
pub struct CreateWalletAddressData {
    mnemonic: String,
    child_number: u64,
}

#[post("/create_wallet_account", data = "<data>")]
pub fn create_wallet_account(data: Form<CreateWalletAddressData>) -> Result<Json<JsonValue>> {
    let mut wallet = WalletLibrary::new_from_mnemonic(Mnemonic::from(&data.mnemonic)?);
    let address = wallet.new_address_at_child_number(ChildNumber::new(data.child_number))?;
    let private_key_bytes = wallet
        .get_child_private_key(ChildNumber::new(data.child_number))?
        .get_private()
        .to_bytes();

    let private_key_hex = hex::encode(private_key_bytes);

    Ok(Json(json!({
        "address": format!("{}", &address),
        "child_number": data.child_number,
        "private_key": private_key_hex,
    })))
}

#[get("/get_latest_account_state/<addr>")]
pub fn get_latest_account_state(
    state: State<AppState>,
    addr: String,
) -> Result<Json<AccountResourceSer>> {
    let address = utils::address_from_strings(&addr)?;
    let account_blob = state.client.get_account_blob(address)?.0;
    let account_resource = utils::get_account_resource_or_default(&account_blob)?;

    Ok(Json(account_resource.into()))
}

#[derive(FromForm)]
pub struct MintCoinsData {
    receiver: String,
    /// In micro libras
    num_coins: u64,
}

#[post("/mint_coins", data = "<data>")]
pub fn mint_coins(state: State<AppState>, data: Form<MintCoinsData>) -> Result<Json<JsonValue>> {
    let receiver = utils::address_from_strings(&data.receiver)?;
    state.faucet_client.mint_coins(&receiver, data.num_coins)?;

    Ok(Json(json!({ "success": true })))
}

#[derive(FromForm)]
pub struct TransferCoinsData {
    sender_addr: String,
    receiver_addr: String,
    num_coins: u64,
    gas_unit_price: Option<u64>,
    max_gas_amount: Option<u64>,

    // authorization
    private_key: Option<String>,
    mnemonic: Option<String>,
    child_number: Option<u64>,
}

#[post("/transfer_coins", data = "<data>")]
pub fn transfer_coins(
    state: State<AppState>,
    data: Form<TransferCoinsData>,
) -> Result<Json<JsonValue>> {
    let mut client =
        Client::from_form_fields(&data.private_key, &data.mnemonic, data.child_number)?;
    let sender = utils::address_from_strings(&data.sender_addr)?;
    let receiver = utils::address_from_strings(&data.receiver_addr)?;

    let result = client.transfer_coins(
        &state,
        sender,
        receiver,
        data.num_coins,
        data.gas_unit_price,
        data.max_gas_amount,
    )?;

    Ok(Json(json!({
        "success": true,
        "sequence": result,
    })))
}

#[get("/get_committed_txn_by_acc_seq/<addr>?<sequence_number>&<fetch_events>")]
pub fn get_committed_txn_by_acc_seq(
    state: State<AppState>,
    addr: String,
    sequence_number: u64,
    fetch_events: bool,
) -> Result<Json<Vec<TxWithEvents>>> {
    let address = utils::address_from_strings(&addr)?;

    state
        .client
        .get_txn_by_acc_seq(address, sequence_number, fetch_events)
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
//
#[get("/get_committed_txn_by_range?<start_version>&<limit>&<fetch_events>")]
pub fn get_committed_txn_by_range(
    state: State<AppState>,
    start_version: u64,
    limit: u64,
    fetch_events: bool,
) -> Result<Json<Vec<TxWithEvents>>> {
    state
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

#[derive(FromFormValue)]
pub enum EventType {
    Sent,
    Received,
}

#[get("/get_events_by_account_and_type/<addr>?<event_type>&<start_seq_number>&<limit>&<ascending>")]
pub fn get_events_by_account_and_type(
    state: State<AppState>,
    addr: String,
    event_type: EventType,
    start_seq_number: u64,
    limit: u64,
    ascending: bool,
) -> Result<Json<AccWithEvents>> {
    let address = utils::address_from_strings(&addr)?;

    let path = match event_type {
        EventType::Sent => account_sent_event_path(),
        EventType::Received => account_received_event_path(),
    };

    let access_path = AccessPath::new(address, path);

    state
        .client
        .get_events_by_access_path(access_path, start_seq_number, ascending, limit)
        .map(|(events, account)| Json(AccWithEvents { account, events }))
        .map_err(|err| From::from(err))
}
