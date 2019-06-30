use chrono::Utc;
use futures::Future;
use futures::stream::Stream;
use protobuf::Message;

use admission_control_proto::proto::admission_control::SubmitTransactionRequest;
use crypto::signing::{KeyPair, sign_message};
use crypto::PrivateKey;
use crypto::hash::CryptoHash;
use failure_ext::prelude::*;
use libra_wallet::{WalletLibrary, Mnemonic};
use libra_wallet::key_factory::{ChildNumber};
use types::{
    transaction::{RawTransaction, SignedTransaction, RawTransactionBytes, Program},
    proto::transaction::SignedTransaction as ProtoSignedTransaction,
};
use proto_conv::{IntoProto, FromProto};
use types::account_address::AccountAddress;


pub enum Client {
    Wallet(WalletLibrary, ChildNumber),
    KeyPair(KeyPair),
}

impl Client {
    pub fn from_mnemonic(mnemonic: &str, child: ChildNumber) -> Result<Self> {
        let mnemonic = Mnemonic::from(mnemonic)?;
        let wallet = WalletLibrary::new_from_mnemonic(mnemonic);

        Ok(Client::Wallet(wallet, child))
    }
    
    pub fn from_private_key(private_key: PrivateKey) -> Self {
        Client::KeyPair(KeyPair::new(private_key))
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
                let raw_bytes = tx.clone().into_proto().write_to_bytes()?;
                let txn_hashvalue = RawTransactionBytes(&raw_bytes).hash();
                let signature = sign_message(txn_hashvalue, pair.private_key())?;
                let public_key = pair.public_key();

                Ok(SignedTransaction::craft_signed_transaction_for_client(tx, public_key, signature))
            }
            Client::Wallet(wallet, child) => {
                wallet.new_address_at_child_number(*child)?;
                wallet.sign_txn(tx)
                    .context("Failed to sign transaction with wallet")
                    .map_err(|err| err.into())
            }
        }
    }
}

// TODO: Support local faucet account
pub struct FaucetClient {
    pub faucet_url: String,
}

impl FaucetClient {
    // TODO(perf): Rewrite in async way (use the global tokio runtime).
    pub fn mint_coins(&self, receiver: &AccountAddress, num_coins: u64) -> Result<()> {
        let mut runtime = tokio::runtime::Runtime::new()?;
        let client = hyper::Client::new();

        let url = format!(
            "http://{}?amount={}&address={:?}",
            self.faucet_url, num_coins, receiver
        )
            .parse::<hyper::Uri>()?;

        let response = runtime.block_on(client.get(url))?;
        let status_code = response.status();
        let body = response.into_body().concat2().wait()?;
        let raw_data = std::str::from_utf8(&body)?;

        if status_code != 200 {
            return Err(format_err!(
                "Failed to query remote faucet server[status={}]: {:?}",
                status_code,
                raw_data,
            ));
        }

//        let sequence_number = raw_data.parse::<u64>()?;
//        if is_blocking {
//            self.wait_for_transaction(AccountAddress::new([0; 32]), sequence_number);
//        }

        Ok(())
    }
}
