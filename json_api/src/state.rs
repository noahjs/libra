use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;
use std::fs;

use parking_lot::RwLock;

use client::AccountData;
use client::grpc_client::GRPCClient;
use failure_ext::prelude::*;
use config::trusted_peers::TrustedPeersConfig;
use types::account_address::AccountAddress;
use crypto::PublicKey;
use types::validator_verifier::ValidatorVerifier;
use crypto::signing::KeyPair;

// TODO: Support local faucet account
pub struct FaucetClient {
    pub faucet_url: String,
}

impl FaucetClient {
    // TODO(perf): Rewrite in async way (use the global tokio runtime).
    pub fn mint_coins(&self, receiver: &AccountAddress, num_coins: u64) -> Result<()> {
        let mut runtime = tokio::Runtime::new().unwrap();
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


pub struct AppState {
    pub client: GRPCClient,
    // Only needed for minting coins for testnet
    pub faucet_client: FaucetClient,
}

impl AppState {
    pub fn new(
        host: &str,
        ac_port: &str,
        validator_set_file: &str,
        faucet_server: Option<String>,
    ) -> Result<Self> {
        let validators_config = TrustedPeersConfig::load_config(Path::new(validator_set_file));
        let validators = validators_config.get_trusted_consensus_peers();
        
        ensure!(
            !validators.is_empty(),
            "Not able to load validators from trusted peers config!"
        );
        
        // Total 3f + 1 validators, 2f + 1 correct signatures are required.
        // If < 4 validators, all validators have to agree.
        let quorum_size = validators.len() * 2 / 3 + 1;
        let validator_verifier = Arc::new(ValidatorVerifier::new(validators, quorum_size));
        let client = GRPCClient::new(host, ac_port, validator_verifier)?;

        let faucet_url = match faucet_server {
            Some(server) => server.to_string(),
            None => host.replace("ac", "faucet"),
        };

        Ok(AppState {
            client,
            faucet_client: FaucetClient {
                faucet_url,
            },
        })
    }
}
