use std::{io::Error, path::Path, convert::TryInto};
use sha2::{Digest, Sha256};
use ed25519_dalek::Signer;

use protocol_types::{Serializable, crypto};

use crate::{setup::{ConfigField, self}, operations::query::http_formatted};

// Submit module handles transaction submit request.
// It formats the request body and assemble the http request for submit.
// It gives recoverable errors if the user input invalid input.


// Submit function format most of the body data require to submit a transaction by http request.
// The only remaining part are hash and signaures.
pub async fn submit(from_address: String, to_address: String, value: u64, tip: u64, gas_limit: u64, gas_price: u64,
data_string: String, deploy_args_string: String, nonce: u64, path_to_keypair_json: String) {
    let tx_result = raw_tx_to_protocol_type_tx(from_address, to_address, value, tip, gas_limit, gas_price,
        data_string, deploy_args_string, nonce, path_to_keypair_json);
    let route_address_raw = setup::read_config(ConfigField::TargetAddress);
    let route_address = &route_address_raw[1..(route_address_raw.len()-1)];
    let client = reqwest::Client::new();


    match tx_result {
        Ok(tx) => {let resp = match client.post(format!("{}/transaction", route_address))
            .body(tx)
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
        },
        Err(e) => {
            println!("The Transaction has wrong format or unexpcted problem encountered. The transaction is not sent.");
            println!("Error: {:?}", e.to_string())
        }
    }
}

// assemble the transaction to a protocol type transaction
fn raw_tx_to_protocol_type_tx(from_address: String, to_address: String, value: u64, tip: u64, gas_limit: u64, gas_price: u64,
    data_string: String, deploy_args_string: String, nonce: u64, path_to_keypair_json: String) ->  Result<Vec<u8>, Error> {
        let sender_address: crypto::PublicAddress = match base64::decode(from_address.clone()){
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
        };
        let receiver_address: crypto::PublicAddress;
        let data: Vec<u8>;

        // To address is null if and only if it is a deploy transaction
        if to_address != "null" {

            ///////////////////////////////////////////////////
            // External to External OR External to Contract //
            //////////////////////////////////////////////////
            
            receiver_address = match base64::decode(to_address.clone()){
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
            };
            data = match base64::decode(data_string.clone()){
                Ok(decode_data) => decode_data,
                Err(_) => {
                    println!("Error: Data field of your input is not basae64 decodable.");
                    std::process::exit(-1);
                }
            };
        } else {

            /////////////////////////
            // Deploy Transaction //
            ///////////////////////
            
            receiver_address = [0; 32];
            let contract_code = match data_string.chars().next() {
                Some(char) => {
                    let char_str = char.to_string();
                    // As required by cli, importing contract by path must start with `./` to indicate a relative path.
                    // Else it will be read as base64 encoded file bytes
                    if char_str == ".".to_string(){
                        println!("Try to retrieve contract from designated path");
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
                        match base64::decode(data_string.clone()){
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

            let deploy_args = match base64::decode(deploy_args_string.clone()){
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

            data = protocol_types::TransactionDataContractDeployment::serialize(&transaction_data);

            let mut hasher = Sha256::new();
            let mut pre_image = Vec::new();
            pre_image.extend(&contract_code);
            pre_image.extend(&sender_address);
            pre_image.extend(nonce.to_be_bytes().to_vec());
    
            hasher.update(pre_image);
    
            let contract_protoaddr = hasher.finalize().to_vec();
            println!("Contract address: {:?}", base64::encode(contract_protoaddr));
        }
    
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

        hash_and_sign_transaction(path_to_keypair_json.clone(), tx_protocol_type)
}
    
/// hash_and_sign_transaction obviously hash and sign transactions and serialize transaction to pchain-types-encoded bytes.
fn hash_and_sign_transaction(path_to_keypair_json: String, mut transaction: protocol_types::transaction::Transaction ) -> Result<Vec<u8>, Error>{
    // Serialize the transaction for signature verification.
    let serialized_transaction = protocol_types::transaction::Transaction::serialize(&transaction);

    // Retrieve the keypair from base64 string
    let keypair_raw = match base64::decode(path_to_keypair_json){
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
    let signature: ed25519_dalek::Signature = keypair.sign(&serialized_transaction[..]);

    // Compute the hash of transaction
    let mut hasher = Sha256::new();
    hasher.update(&signature);
    let computed_hash = hasher.finalize();

    println!("Hash of tx: {:?}", base64::encode(&computed_hash));

    transaction.hash = computed_hash.into();
    transaction.signature = signature.to_bytes();

    let serialized_transaction_final = protocol_types::Transaction::serialize(&transaction);
    Ok(serialized_transaction_final)
}
    
