//! Implementations of [Serialize](serde::Serialize) for various remote types.
use serde::{ser::Serialize, Serializer};

use crypto::HashValue;
use types::{
    access_path::AccessPath,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::{ContractEvent, EventWithProof},
    proof::{AccountStateProof, AccumulatorProof, EventProof, SparseMerkleProof},
    transaction::{SignedTransaction, TransactionInfo, Version},
    account_config::AccountResource,
    byte_array::ByteArray,
};

// Pure insanity. Might be better to add derive(Serialize) to all definitions.

#[derive(Serialize)]
pub struct AccountResourceSer {
    pub balance: u64,
    pub sequence_number: u64,
    pub authentication_key: ByteArray,
    pub sent_events_count: u64,
    pub received_events_count: u64,
}

impl From<AccountResource> for AccountResourceSer {
    fn from(acc: AccountResource) -> Self {
        AccountResourceSer {
            balance: acc.balance(),
            sequence_number: acc.sequence_number(),
            authentication_key: acc.authentication_key().clone(),
            sent_events_count: acc.sent_events_count(),
            received_events_count: acc.received_events_count(),
        }
    }
}

#[derive(Serialize)]
pub struct TxWithEvents {
    pub transaction: SignedTransaction,
    #[serde(serialize_with = "serialize_contract_events")]
    pub events: Option<Vec<ContractEvent>>,
}

pub fn serialize_contract_events<S>(
    value: &Option<Vec<ContractEvent>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    struct Helper<'a>(#[serde(with = "ContractEventSer")] &'a ContractEvent);

    value
        .as_ref()
        .map(|vec| vec.iter().map(Helper).collect::<Vec<_>>())
        .serialize(serializer)
}

#[derive(Serialize)]
pub struct AccWithEvents {
    #[serde(serialize_with = "serialize_account")]
    pub account: Option<AccountStateWithProof>,
    #[serde(serialize_with = "serialize_events_with_proof")]
    pub events: Vec<EventWithProof>,
}

pub fn serialize_account<S>(
    value: &Option<AccountStateWithProof>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    struct Helper<'a>(#[serde(with = "AccountStateWithProofSer")] &'a AccountStateWithProof);

    value.as_ref().map(Helper).serialize(serializer)
}

pub fn serialize_events_with_proof<S>(
    value: &Vec<EventWithProof>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    struct Helper<'a>(#[serde(with = "EventWithProofSer")] &'a EventWithProof);

    value
        .iter()
        .map(Helper)
        .collect::<Vec<_>>()
        .serialize(serializer)
}

#[derive(Serialize)]
#[serde(remote = "AccountStateWithProof")]
pub struct AccountStateWithProofSer {
    pub version: Version,
    #[serde(serialize_with = "serialize_blob")]
    pub blob: Option<AccountStateBlob>,
    #[serde(with = "AccountStateProofSer")]
    pub proof: AccountStateProof,
}

pub fn serialize_blob<S>(value: &Option<AccountStateBlob>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value.as_ref().map(|val| val.as_ref()).serialize(serializer)
}

#[derive(Serialize)]
#[serde(remote = "AccountStateProof")]
pub struct AccountStateProofSer {
    #[serde(
        getter = "AccountStateProof::ledger_info_to_transaction_info_proof",
        with = "AccumulatorProofSer"
    )]
    ledger_info_to_transaction_info_proof: AccumulatorProof,
    #[serde(getter = "AccountStateProof::transaction_info")]
    transaction_info: TransactionInfo,
    #[serde(
        getter = "AccountStateProof::transaction_info_to_account_proof",
        with = "SparseMerkleProofSer"
    )]
    transaction_info_to_account_proof: SparseMerkleProof,
}

#[derive(Serialize)]
#[serde(remote = "SparseMerkleProof")]
pub struct SparseMerkleProofSer {
    #[serde(getter = "SparseMerkleProof::leaf")]
    leaf: Option<(HashValue, HashValue)>,
    #[serde(getter = "SparseMerkleProofSer::siblings_cloned")]
    siblings: Vec<HashValue>,
}

impl SparseMerkleProofSer {
    fn siblings_cloned(value: &SparseMerkleProof) -> Vec<HashValue> {
        value.siblings().to_owned()
    }
}

#[derive(Serialize)]
#[serde(remote = "ContractEvent")]
pub struct ContractEventSer {
    #[serde(getter = "ContractEvent::access_path")]
    pub access_path: AccessPath,
    #[serde(getter = "ContractEvent::sequence_number")]
    pub sequence_number: u64,
    #[serde(getter = "ContractEventSer::event_data_cloned")]
    pub event_data: Vec<u8>,
}

impl ContractEventSer {
    fn event_data_cloned(ev: &ContractEvent) -> Vec<u8> {
        ev.event_data().to_owned()
    }
}

#[derive(Serialize)]
#[serde(remote = "EventWithProof")]
pub struct EventWithProofSer {
    pub transaction_version: u64, // Should be `Version`, but FromProto derive won't work that way.
    pub event_index: u64,
    #[serde(with = "ContractEventSer")]
    pub event: ContractEvent,
    #[serde(with = "EventProofSer")]
    pub proof: EventProof,
}

#[derive(Serialize)]
#[serde(remote = "EventProof")]
pub struct EventProofSer {
    #[serde(
        with = "AccumulatorProofSer",
        getter = "EventProof::ledger_info_to_transaction_info_proof"
    )]
    pub ledger_info_to_transaction_info_proof: AccumulatorProof,
    #[serde(getter = "EventProof::transaction_info")]
    pub transaction_info: TransactionInfo,
    #[serde(
        with = "AccumulatorProofSer",
        getter = "EventProof::transaction_info_to_event_proof"
    )]
    pub transaction_info_to_event_proof: AccumulatorProof,
}

#[derive(Serialize)]
#[serde(remote = "AccumulatorProof")]
pub struct AccumulatorProofSer {
    #[serde(getter = "AccumulatorProofSer::siblings_cloned")]
    pub siblings: Vec<HashValue>,
}

impl AccumulatorProofSer {
    fn siblings_cloned(ev: &AccumulatorProof) -> Vec<HashValue> {
        ev.siblings().to_owned()
    }
}
