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
use std::{convert::TryInto, fs};

use crate::{display_types::*, setup::{ConfigField, self}, args_parser};
use protocol_types::Deserializable;

// Query module handles all query request.
// It formats the query string and assemble the target address for query.
// It also format the response and print it to human readble form.

pub enum QueryOption{
    BlocksByBlockHash,
    BlocksByBlockNum,
    BlocksByTxHash,
    BlocksLatest,

    TxsByTxHash,
    TxsByTxNum,
    TxsLatest,

    View,
    Balance,
    Nonce,
    ContractCode,
    ContractMetadata,
    WorldState,
    TxProof,
    ReceiptProof,
    Mempoolsize,

    GasPerBlock,
    MempoolSize,
    None,
}

pub(crate) struct Api {
    standard: String,
    rich: String,
    analytics: String,
}

// Query funtion map each different kind of queries to their corresponding endpoint.
pub async fn query(query_option: QueryOption, query_inputs: Vec<String>){
    // Retrive target host address to query upon
    let client = reqwest::Client::new();
    //let route_address = setup::read_config(ConfigField::RichApiUrl);

    let api = Api {
        standard: setup::read_config(ConfigField::TargetUrl),
        rich: setup::read_config(ConfigField::RichApiUrl),
        analytics: setup::read_config(ConfigField::AnalyticsApiUrl),
    };

    //let route_address = route_address_raw.as_str();// &route_address_raw[1..(route_address_raw.len()-1)];

    match query_option {
        /////////////////////////
        // Rich Rest APIs
        /////////////////////////
        
        QueryOption::BlocksByBlockHash | QueryOption::BlocksByBlockNum | QueryOption::BlocksByTxHash | QueryOption::BlocksLatest => {
            let selector_argument = &query_inputs[0];
            let window = &query_inputs[1];
            let header_only = &query_inputs[2];
            let summary_only = &query_inputs[3];

            let mut args = vec![
                format!("window={}",window),
                format!("header_only={}",header_only),
            ];

            match query_option {
                QueryOption::BlocksByBlockHash => args.push(format!("hash={}",selector_argument)),
                QueryOption::BlocksByBlockNum => args.push(format!("num={}",selector_argument)),
                QueryOption::BlocksByTxHash => args.push(format!("tx_hash={}",selector_argument)),
                QueryOption::BlocksLatest => args.push(format!("latest={}", selector_argument)),
                _=> {}
            }

            if !summary_only.is_empty() {
                args.push(format!("summary_only={}", summary_only));
            }

            let route = format!("{}/blocks?{}", &api.rich, args.join("&"));
            
            let query_type = 
            if summary_only == &"true".to_string() {
                QueryReturnType::BlockSummary
            } else if header_only == &"true".to_string() {
                QueryReturnType::BlockHeaders 
            } else { 
                QueryReturnType::Blocks
            };

            query_helper(route, None, query_type, client).await;
        },
        QueryOption::TxsByTxHash | QueryOption::TxsByTxNum | QueryOption::TxsLatest => {
            let selector_argument = &query_inputs[0];
            let window = &query_inputs[1];
            let summary_only = &query_inputs[2];

            let mut args = vec![
                format!("window={}",window),
            ];

            match query_option {
                QueryOption::TxsByTxHash => args.push(format!("tx_hash={}",selector_argument)),
                QueryOption::TxsByTxNum => args.push(format!("num={}",selector_argument)),
                QueryOption::TxsLatest => args.push(format!("latest={}",selector_argument)),
                _=> {}
            }
            
            if !summary_only.is_empty() {
                args.push(format!("summary_only={}", summary_only));
            }

            let route = format!("{}/transactions?{}", &api.rich, args.join("&"));

            let query_type = 
            if summary_only == &"true".to_string() {
                QueryReturnType::TransactionSummary
            } else {
                QueryReturnType::Transactions
            };

            query_helper(route, None, query_type, client).await;
        },
        /////////////////////////
        // Standard Rest APIs
        /////////////////////////

        QueryOption::Balance => {
            let address = &query_inputs[0];
            let route = format!("{}/account/{}/balance?proof=false", &api.standard, address);
            query_helper(route, None, QueryReturnType::NumberU64, client).await;
        },
        QueryOption::Nonce => {
            let address = &query_inputs[0];
            let route = format!("{}/account/{}/nonce?proof=false", &api.standard, address);
            query_helper(route, None, QueryReturnType::NumberU64, client).await;
        },
        QueryOption::ContractCode => {
            let address = &query_inputs[0];
            let route = format!("{}/account/{}/code?proof=false", &api.standard, address);
            query_helper(route, None, QueryReturnType::Binary, client).await;
        },
        QueryOption::ContractMetadata => {
            let address = &query_inputs[0];
            let route = format!("{}/account/{}/metadata", &api.standard, address);
            query_helper(route, None, QueryReturnType::Text, client).await;
        }
        QueryOption::WorldState => {
            let address = &query_inputs[0];
            let key = &query_inputs[1];
            let route = format!("{}/account/{}/state?key={}&proof=false", &api.standard, address, key);
            query_helper(route, None, QueryReturnType::Text, client).await;
        },
        QueryOption::View => {
            let address = &query_inputs[0];
            let calldata = match protocol_types::Base64URL::decode(&query_inputs[1]) {
                Ok(d) => d,
                Err(e) =>{
                    println!("Illegal input for call data. {:?}.", e);
                    std::process::exit(1);  
                }
            };
            let expected_callback = &query_inputs[2];

            let route = format!("{}/account/{}/view", &api.standard, address);
            let encoded_callback = query_helper(route, Some(calldata), QueryReturnType::Callback, client).await;
            if expected_callback != &"".to_string() {
                println!("Your callback value (parsed) {}", args_parser::from_callback(encoded_callback, expected_callback.clone()));
            }
        },
        QueryOption::TxProof => {
            let block_hash = &query_inputs[0];
            let tx_hash = &query_inputs[1];
            let route = format!("{}/proof/transaction/block/{}/transaction/{}", &api.standard, block_hash, tx_hash);
            query_helper(route, None, QueryReturnType::Text, client).await;
        },
        QueryOption::ReceiptProof => {
            let block_hash = &query_inputs[0];
            let tx_hash = &query_inputs[1];

            let route = format!("{}/proof/receipt/block/{}/transaction/{}", &api.standard, block_hash, tx_hash);
            query_helper(route, None, QueryReturnType::Text, client).await;
        },
        QueryOption::Mempoolsize => {
            let route = format!("{}/mempoolsize", &api.standard);
            query_helper(route, None, QueryReturnType::NumberU64, client).await;
        }

        /////////////////////////
        // Analytics Rest APIs
        /////////////////////////

        QueryOption::GasPerBlock |  QueryOption::MempoolSize => {
            let converted_inputs: Vec<u64> = query_inputs.iter().map(|q|{
                match q.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Illegal input. It should be a number");
                        std::process::exit(1); 
                    }
                }
            }).collect();
            let from_time:u64 = converted_inputs[0];
            let to_time:u64 = converted_inputs[1];
            let window_size:u64 = converted_inputs[2];
            let step_size:u64 =  converted_inputs[3];

            let category = match query_option {
                QueryOption::GasPerBlock => "gas_per_block",
                QueryOption::MempoolSize => "mempool_size",
                _ => unreachable!()
            };

            let route = format!("{}/{}?from_time={}&to_time={}&window_size={}&step_size={}", &api.analytics, category, from_time, to_time, window_size, step_size);
            query_helper(route, None, QueryReturnType::VectorU64, client).await;
        },
        QueryOption::None => {println!("You should not reach here.")},
    }
}

enum QueryReturnType{
    Blocks,
    BlockHeaders,
    BlockSummary,
    Transactions,
    TransactionSummary,
    VectorU64,
    Binary,
    Text,
    NumberU64,
    Callback,
}

// Query helper sned the request and helps to transalte the return result from restAPI endpoints to more human readable content.
// e.g. Bock and Transaction which is in protobuf bytes will be formatted and beautified.
// e.g. Analytics result which should be in vector of u64 return comes in vector of u8 when return from API. We also had to transalte it back. 
async fn query_helper(route: String, data: Option<Vec<u8>>, query_return_type:QueryReturnType, client: reqwest::Client) -> String{

    let get_future = if data.is_some() {
        client.get(route).body(data.unwrap()).send()
    } else {
        client.get(route).send()
    };

    let resp = match get_future.await {
        Ok(some_resp) => some_resp,
        Err(e) => {
            println!("Error: Server connection error");
            println!("Detail: {}", format!("{}", e));
            std::process::exit(1);
        }
    };

    if resp.status().is_success() {
        match query_return_type {
            QueryReturnType::Blocks => {
                let blocks_return = Vec::<protocol_types::Block>::deserialize(&resp.bytes().await.unwrap()).unwrap();
                let blocks_print: Blocks = From::<  Vec::<protocol_types::Block> >::from(blocks_return);
                println!("Your Blocks: {:#?}", blocks_print);
            },
            QueryReturnType::BlockHeaders => {
                let blocks_return = Vec::<protocol_types::BlockHeader>::deserialize(&resp.bytes().await.unwrap()).unwrap();
                let blocks_print: BlockHeaders = From::<  Vec::<protocol_types::BlockHeader> >::from(blocks_return);
                println!("Your Blocks (header only): {:#?}", blocks_print);
            },
            QueryReturnType::BlockSummary => {
                let value = &resp.text().await.unwrap();
                let block_summarys: Vec<BlockSummary> = serde_json::from_str(value.as_str()).unwrap();
                println!("Your Block Summary: {:#?}", block_summarys);
            },
            QueryReturnType::Transactions => {
                let tx_return = Vec::<(u64, protocol_types::Transaction, protocol_types::Receipt)>::deserialize(&resp.bytes().await.unwrap()).unwrap();
                let tx_print: TransactionsWithReceipt = From::< Vec::<(u64, protocol_types::Transaction, protocol_types::Receipt)> >::from(tx_return);
                println!("Your Txs: {:#?}", tx_print);
            },
            QueryReturnType::TransactionSummary => {
                let value = &resp.text().await.unwrap();
                let txn_summarys: Vec<TxnSummary> = serde_json::from_str(value.as_str()).unwrap();
                println!("Your Txn Summary: {:#?}", txn_summarys);
            },
            QueryReturnType::VectorU64 => {
                let value = &resp.bytes().await.unwrap().to_vec();
                let u64vec_from_u8bytes:Vec<u64> = (0..value.len()/8).map(
                    |index|{
                        u64::from_le_bytes(value[8*index..8*(index+1)].try_into().unwrap())
                    }
                ).collect();
                println!("Your value {:?}", u64vec_from_u8bytes);
            },
            QueryReturnType::Binary => {
                let value = &resp.bytes().await.unwrap().to_vec();
                match fs::write("contract-code.bin",value) {
                    Ok(_) => {
                        println!("Your result is saved to binary file `contract-code.bin` in same directory");
                    },
                    Err(_) => {
                        println!("Unable to save result to binary file (size: {} bytes). ", value.len());
                    }
                }
            }
            QueryReturnType::Text => {
                let value = &resp.text().await.unwrap();
                let decoded_value = match protocol_types::Base64URL::decode(value){
                    Ok(decode) => decode,
                    Err(_) => {
                        println!("Your value is: {:?}", value);
                        std::process::exit(1);
                    }
                };
                println!("Your value {:?}", value);
                println!("Your value(decoded) {:?}", decoded_value);
            }
            QueryReturnType::NumberU64 => {
                let value = &resp.bytes().await.unwrap().to_vec();
                let numeric_value = {
                    let mut buf = [0u8; 8];
                    buf.copy_from_slice(value.as_slice());
                    u64::from_le_bytes(buf)
                };
                println!("Your value {}", numeric_value);
            }
            QueryReturnType::Callback => {
                let value = &resp.bytes().await.unwrap().to_vec();

                let encoded = protocol_types::Base64URL::encode(value);
                println!("Your callback value (encoded): {}\n", encoded.to_string());

                args_parser::from_callback(encoded.to_string(), "data_type".to_string());

                if let Ok(s) = String::from_utf8(value.clone()) {
                    println!("Your value (utf8 representation): {}", s);
                }

                return encoded.to_string()
            }
        }
    }else {
        println!("Query Error. Status: {:?}", resp.status());
        let resp_detail = resp.text().await.unwrap();
        println!("Query Error. Detail: {}", http_formatted(resp_detail));
    } 
    return "".to_string()
}

// Http formatted beautified the html string return from server when it encounter errors.
// It remove all the html tags and displace html in beautiful, human readble format.
pub(crate) fn http_formatted(resp_detail: String) -> String {
    let mut resp_formatted: String = String::new();
    if resp_detail.starts_with("<!DOCTYPE html>") {
        // skip all tags
        let mut within_html_tag_flag = false;
        resp_detail.chars().for_each(
            |char| {
                if char == '<' {within_html_tag_flag = true}
                else if char == '>' {within_html_tag_flag = false}
                else if within_html_tag_flag == false {
                    resp_formatted.push(char);
                }
            }
        );
    } else {
        resp_formatted = resp_detail;
    };
    return resp_formatted;
}