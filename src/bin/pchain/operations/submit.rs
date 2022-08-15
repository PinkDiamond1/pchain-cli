/*
 Copyright (c) 2022 ParallelChain Lab

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::{io::Error, path::Path, convert::TryInto};
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use ed25519_dalek::Signer;

use protocol_types::{Serializable, crypto, PublicAddress};

use crate::{setup::{ConfigField, self}, operations::{query::http_formatted, KeypairJSON}, Base64String};

// Submit module handles transaction submit request.
// It formats the request body and assemble the http request for submit.
// It gives recoverable errors if the user input invalid input.


// Submit function format most of the body data require to submit a transaction by http request.
// The only remaining part are hash and signaures.
pub async fn submit(tx_json: SubmitTxJson, is_deploy: bool) {
    let tx_json_string = serde_json::to_string_pretty(&tx_json).unwrap();

    let sender_address = parse_sender_address(tx_json.from_address);
    let (receiver_address, data) = if is_deploy { 
        let (contract_address, data) = parse_contract(sender_address, tx_json.nonce, tx_json.data, tx_json.deploy_args);
        println!("Contract address: \"{}\"",contract_address);
        ([0u8;32], data)
    } else {
        (parse_eoa_receiver_address(tx_json.to_address.clone()), parse_tx_data(tx_json.data))
    };
    let tx_data = match build_protocol_types_tx(
        sender_address, 
        receiver_address, 
        tx_json.value, 
        tx_json.tip, 
        tx_json.gas_limit, 
        tx_json.gas_price, 
        data, 
        tx_json.nonce, 
        tx_json.path_to_keypair_json) {
            Ok(tx_data) => tx_data,
            Err(e) => {
                println!("The Transaction has wrong format or unexpcted problem encountered. The transaction is not sent.");
                println!("Error: {:?}", e.to_string());
                return;
            }
        };

    let route_address_raw = setup::read_config(ConfigField::TargetUrl);
    let route_address = route_address_raw;
    let client = reqwest::Client::new();
    let api_url = format!("{}/transaction", route_address);
    println!("Submit Transaction {}", tx_json_string);
    post_transaction(client, tx_data, api_url).await;
}

async fn post_transaction(client: reqwest::Client, tx_data: Vec<u8>, api_url: String) {
    let resp = match client.post(api_url)
    .body(tx_data)
    .send()
    .await{
        Ok(some_resp) => some_resp,
        Err(e) => {
            println!("Error: Server connection error");
            println!("Detail: {}", format!("{}", e));
            std::process::exit(1);
        }
    };
    if resp.status().is_success() {
        println!("Status {:#?}", resp.status());
        println!("Response {:#?}", resp.text().await.unwrap());
    } else {
        println!("Submit Error. Status: {:?}", resp.status());
        let resp_detail = resp.text().await.unwrap();
        println!("Submit Error. Detail: {}", http_formatted(resp_detail));
    }
}

fn parse_sender_address(from_address: String) -> PublicAddress {
    match protocol_types::Base64URL::decode(&from_address.clone()){
        Ok(address) => match address.try_into() {
            Ok(address) => address,
            Err(e) => {
                println!("Error: 'from_address' must be 32 bytes long: {:?}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            println!("Error: Wrong encoding of to address: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn parse_eoa_receiver_address(to_address: String) -> PublicAddress{
    match protocol_types::Base64URL::decode(&to_address.clone()){
        Ok(address) => match address.try_into() {
            Ok(address) => address,
            Err(e) => {
                println!("Error: 'to_address' has to be 32 bytes long: {:?}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            println!("Error: Wrong encoding of to address: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn parse_tx_data(data_string: String) -> Vec<u8> {
    match protocol_types::Base64URL::decode(&data_string.clone()){
        Ok(decode_data) => decode_data,
        Err(_) => {
            println!("Error: Data field of your input is not basae64 decodable.");
            std::process::exit(-1);
        }
    }
}

fn parse_contract(sender_address: crypto::PublicAddress, nonce: u64, data_string: String, deploy_args_string: String) -> (Base64String, Vec<u8>) {
    let contract_address: Base64String;
    let contract_code = match data_string.chars().next() {
        Some(char) => {
            let char_str = char.to_string();
            // As required by cli, importing contract by path must start with `./` to indicate a relative path.
            // Else it will be read as base64 encoded file bytes
            if char_str == ".".to_string(){
                if Path::new(&data_string).is_file(){
                    match std::fs::read(&data_string) {
                        Ok(data) => data,
                        Err(e) => {
                            println!("Error: : Fail to read designated file although is file found {:?}", e);
                            std::process::exit(1);  
                        }
                    }
                } else {
                    println!("Error: : Invalid path. Cannot retrieve the smart contract from the designated path.");
                    std::process::exit(1);
                }
            } else {
                match protocol_types::Base64URL::decode(&data_string.clone()){
                    Ok(tx_data) => tx_data,
                    Err(e) => {
                        println!("Error: Wrong encoding of data: {:?}", e);
                        println!("Notice: If you want to input file by path, you must use relative path and start with `./` e,g, ./some_directory/some_sc.wasm`");
                        std::process::exit(1);
                    }
                }
            }
        },
        None => {println!("Error: Data is required for Deploy transaction"); std::process::exit(1)}
    };

    let deploy_args = match protocol_types::Base64URL::decode(&deploy_args_string.clone()){
        Ok(decode_data) => decode_data,
        Err(_) => {
            println!("Error: deploy-args of your input is not basae64 decodable.");
            std::process::exit(-1);
        }
    };

    // Data in protocol_types::transaction will be seperated into contract bytecodes and arguments for 'init' entrypoint of the contract
    let transaction_data = protocol_types::transaction::TransactionDataContractDeployment {
        contract_code: contract_code.clone(),
        contract_init_arguments: deploy_args
    };

    let data = protocol_types::TransactionDataContractDeployment::serialize(&transaction_data);

    let mut hasher = Sha256::new();
    let mut pre_image = Vec::new();
    pre_image.extend(&contract_code);
    pre_image.extend(&sender_address);
    pre_image.extend(nonce.to_le_bytes().to_vec());

    hasher.update(pre_image);

    let contract_protoaddr = hasher.finalize().to_vec();
    contract_address = protocol_types::Base64URL::encode(contract_protoaddr).to_string();
    (contract_address, data)
    // println!("Contract address: {:?}", contract_address);
    
}

fn build_protocol_types_tx(
    sender_address: crypto::PublicAddress, 
    receiver_address: crypto::PublicAddress, 
    value: u64, 
    tip: u64, 
    gas_limit: u64, 
    gas_price: u64,
    data: Vec<u8>,
    nonce: u64, 
    path_to_keypair_json: String) 
    ->  Result<Vec<u8>, Error> {
    let tx_protocol_type = protocol_types::transaction::Transaction {
        from_address: sender_address,
        to_address: receiver_address,
        value: value,
        tip: tip,
        gas_limit: gas_limit,
        gas_price: gas_price,
        data,
        n_txs_on_chain_from_address: nonce,
        // the hash and fields are expected to be populated. Otherwise the node will not
        // accept the transaction.
        hash: [0; 32],
        signature: [0; 64],
    }; 

    let keypair_base64_string = load_keypair(path_to_keypair_json);

    hash_and_sign_transaction(keypair_base64_string, tx_protocol_type)
}
    
/// hash_and_sign_transaction obviously hash and sign transactions and serialize transaction to pchain-types-encoded bytes.
fn hash_and_sign_transaction(keypair_base64_string: String, mut transaction: protocol_types::transaction::Transaction ) -> Result<Vec<u8>, Error>{
    // Retrieve the keypair from base64 string
    let keypair_raw = match protocol_types::Base64URL::decode(&keypair_base64_string){
        Ok(address) => address,
        Err(e) => {
            println!("Error: Wrong encoding of keypair: {:?}", e);
            std::process::exit(1);
        }
    };
    let keypair:ed25519_dalek::Keypair = match ed25519_dalek::Keypair::from_bytes(&keypair_raw){
        Ok(key) => key,
        Err(e) => {
            println!("Invalid keypair though the encoding is right: {:?}", e);
            std::process::exit(1);
        }
    };

    // Serialize the transaction for signature verification.
    let serialized_transaction = protocol_types::transaction::Transaction::serialize(&transaction);

    let signature: ed25519_dalek::Signature = keypair.sign(&serialized_transaction[..]);
    println!("Signature of tx: {:?}", protocol_types::Base64URL::encode(&signature).to_string());

    // Compute the hash of transaction
    let mut hasher = Sha256::new();
    hasher.update(&signature);
    let computed_hash = hasher.finalize();

    println!("Hash of tx: {:?}", protocol_types::Base64URL::encode(&computed_hash).to_string());

    transaction.hash = computed_hash.into();
    transaction.signature = signature.to_bytes();

    let output_serialize_tx_data = protocol_types::Transaction::serialize(&transaction);

    Ok(output_serialize_tx_data)
}

fn load_keypair(path_to_keypair_json: String) -> String {
    let keypair_base64_string = if Path::new(&path_to_keypair_json).is_file(){
        match std::fs::read(&path_to_keypair_json) {
            Ok(data) => match String::from_utf8(data) {
                Ok(keypair_json) => {
                    match serde_json::from_str::<KeypairJSON>(keypair_json.as_str()){
                        Ok(kp_json) => kp_json.keypair,
                        Err(e) => {
                            println!("Error: : Fail to parse designated keypair file from json {:?}", e);
                            std::process::exit(1);
                        }
                    }
                },
                Err(e) => {
                    println!("Error: : Fail to parse designated keypair file although is file found {:?}", e);
                    std::process::exit(1);  
                }
            },
            Err(e) => {
                println!("Error: : Fail to read designated keypair file although is file found {:?}", e);
                std::process::exit(1);  
            }
        }
    } else {
        println!("Error: : Invalid path. Cannot retrieve designated keypair file from the designated path.");
        std::process::exit(1);
    };
    keypair_base64_string
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SubmitTxJson {
    pub from_address: String,
    pub to_address: String,
    pub value: u64,
    pub tip: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub data: String,
    pub deploy_args: String,
    pub nonce: u64,
    pub path_to_keypair_json: String
}

impl SubmitTxJson {
    pub fn load_tx_json_file(path_to_json: String) -> SubmitTxJson{
        let tx_json = if Path::new(&path_to_json).is_file(){
            match std::fs::read(&path_to_json) {
                Ok(data) => match String::from_utf8(data) {
                    Ok(keypair_json) => {
                        match serde_json::from_str::<SubmitTxJson>(keypair_json.as_str()){
                            Ok(kp_json) => kp_json,
                            Err(e) => {
                                println!("Error: : Fail to parse tx json file from json {:?}", e);
                                std::process::exit(1);
                            }
                        }
                    },
                    Err(e) => {
                        println!("Error: : Fail to parse tx json file although is file found {:?}", e);
                        std::process::exit(1);  
                    }
                },
                Err(e) => {
                    println!("Error: : Fail to read tx json file although is file found {:?}", e);
                    std::process::exit(1);  
                }
            }
        } else {
            println!("Error: : Invalid path. Cannot retrieve tx json file from the designated path.");
            std::process::exit(1);
        };
        tx_json
    }
}