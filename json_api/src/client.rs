use chrono::Utc;

use admission_control_proto::proto::admission_control::SubmitTransactionRequest;
use crypto::signing::{KeyPair, sign_message};
use crypto::PrivateKey;
use crypto::hash::CryptoHash;
use client::AccountData;
use failure_ext::Result;
use libra_wallet::{WalletLibrary, Mnemonic};
use libra_wallet::key_factory::{ChildNumber, ExtendedPrivKey};
use proto_conv::IntoProto;
use types::{
    transaction::{RawTransaction, SignedTransaction, RawTransactionBytes, Program},
    proto::transaction::SignedTransaction as ProtoSignedTransaction,
};

use crate::state::AppState;
use types::account_address::AccountAddress;

pub enum Client {
    Wallet(WalletLibrary, ChildNumber),
    KeyPair(KeyPair),
}

impl Client {
    pub fn from_mnemonic(mnemonic: &str, child: ChildNumber) -> Result<Self> {
        let mnemonic = Mnemonic::from(mnemonic)?;
        let wallet = WalletLibrary::new_from_mnemonic(mnemonic);

        Ok(Client::Wallet(WalletLibrary, child))
    }
    
    pub fn from_private_key(private_key: PrivateKey) -> Self {
        Ok(Client::KeyPair(KeyPair::new(private_key)))
    }

    /// Craft a transaction request.
    pub fn create_submit_transaction_req(
        &mut self,
        sender_address: AccountAddress,
        sender_sequence_number: u64,
        program: Program,
        gas_unit_price: Option<u64>,
        max_gas_amount: Option<u64>,
    ) -> Result<SubmitTransactionRequest> {
        const GAS_UNIT_PRICE: u64 = 0;
        const MAX_GAS_AMOUNT: u64 = 10_000;
        const TX_EXPIRATION: i64 = 100;
        
        let raw_txn = RawTransaction::new(
            sender_address,
            sender_sequence_number,
            program,
            max_gas_amount.unwrap_or(MAX_GAS_AMOUNT),
            gas_unit_price.unwrap_or(GAS_UNIT_PRICE),
            std::time::Duration::new((Utc::now().timestamp() + TX_EXPIRATION) as u64, 0),
        );

        let signed_txn = self.sign_txn(raw_txn)?;

        let mut req = SubmitTransactionRequest::new();
        req.set_signed_txn(signed_txn.into_proto());
        
        Ok(req)
    }
    
    pub fn sign_txn(&mut self, tx: RawTransaction) -> Result<SignedTransaction> {
        match self {
            Client::KeyPair(pair) => {
                let raw_bytes = tx.into_proto().write_to_bytes()?;
                let txn_hashvalue = RawTransactionBytes(&raw_bytes).hash();
                let signature = sign_message(txn_hashvalue, pair.private_key());
                let public_key = pair.public_key();

                let mut signed_txn = ProtoSignedTransaction::new();
                signed_txn.set_raw_txn_bytes(raw_bytes.to_vec());
                signed_txn.set_sender_public_key(public_key.to_bytes().to_vec());
                signed_txn.set_sender_signature(signature.to_bytes().to_vec());

                Ok(SignedTransaction::from_proto(signed_txn)?)
            }
            Client::Wallet(mut wallet, child) => {
                wallet.new_address_at_child_number(*child)?;
                wallet.sign_txn(tx)
            }
        }
    }
}
