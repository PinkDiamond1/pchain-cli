use std::convert::TryInto;

use crate::{display_types::*, setup::{ConfigField, self}};
use protocol_types::Deserializable;

// Query module handles all query request.
// It formats the query string and assemble the target address for query.
// It also format the response and print it to human readble form.

pub enum QueryOption{
    TxByTxHash,
    LatestBlock,
    BlockByBlockNum,
    BlockByTxHash,
    BlockByBlockHash,
    BlocksByBlockHash,
    BlockNumByBlockHash,
    Balance,
    Nonce,
    ContractCode,
    ContractMetadata,
    WorldState,
    GasPerBlock,
    MempoolSize,
    None,
}

// Query funtion map each different kind of queries to their corresponding endpoint.
pub async fn query(query_option: QueryOption, query_input: Option<&str>){
    // Retrive target host address to query upon
    let client = reqwest::Client::new();
    let route_address_raw = setup::read_config(ConfigField::TargetAddress);
    let route_address = &route_address_raw[1..(route_address_raw.len()-1)];

    match query_option {
        QueryOption::TxByTxHash => {
            if let Some(hash) = query_input {
                let route = format!("{}/transaction?tx_hash={}", route_address, correct_url(hash));
                query_helper(route, QueryReturnType::Transaction, client).await;
            }
        },
        QueryOption::LatestBlock => {
            let route = format!("{}/block", route_address);
            query_helper(route, QueryReturnType::Block, client).await;
        },
        QueryOption::BlockByBlockNum => {
            if let Some(block_num) = query_input {
                let route = format!("{}/block?height={}", route_address, block_num);
                query_helper(route, QueryReturnType::Block, client).await;
            }
        },
        QueryOption::BlockByTxHash => {
            if let Some(hash) = query_input {
                let route = format!("{}/block?tx_hash={}", route_address, correct_url(hash));
                query_helper(route, QueryReturnType::Block, client).await;
            }
        },
        QueryOption::BlockByBlockHash => {
            if let Some(hash) = query_input {
                let route = format!("{}/block?block_hash={}", route_address, correct_url(hash));
                query_helper(route, QueryReturnType::Block, client).await;
            }
        },
        QueryOption::BlocksByBlockHash => {
            // We use `&` character as delimiter of input vector. If people enter illegal character - `&`, if will be checked by number of segments seperated by `&`
            if let Some(input) = query_input {
                let input_vector: Vec<&str> = input.split("&").collect();
                let hash = input_vector[0];
                let size = correct_url(input_vector[1]);
                let route = format!("{}/blocks?block_hash={}&size={}", route_address, correct_url(hash), size);
                query_helper(route, QueryReturnType::Blocks, client).await;
            }
        },
        QueryOption::BlockNumByBlockHash => {
            if let Some(hash) = query_input {
                let route = format!("{}/blocknum?block_hash={}", route_address, correct_url(hash));
                query_helper(route, QueryReturnType::Text, client).await;
            }
        },
        QueryOption::Balance => {
            if let Some(address) = query_input {
                let route = format!("{}/account/balance?address={}", route_address, correct_url(address));
                query_helper(route, QueryReturnType::Text, client).await;
            }
        },
        QueryOption::Nonce => {
            if let Some(address) = query_input {
                let route = format!("{}/account/nonce?address={}", route_address, correct_url(address));
                query_helper(route, QueryReturnType::Text, client).await;
            }
        },
        QueryOption::ContractCode => {
            if let Some(address) = query_input {
                let route = format!("{}/account/contract_code?address={}", route_address, correct_url(address));
                query_helper(route, QueryReturnType::Text, client).await;
            }
        },
        QueryOption::ContractMetadata => {
            if let Some(address) = query_input {
                let route = format!("{}/account/contract_metadata?address={}", route_address, correct_url(address));
                query_helper(route, QueryReturnType::Text, client).await;
            }
        }
        QueryOption::WorldState => {
            if let Some(input) = query_input {
                // We use `&` character as delimiter of input vector. If people enter illegal character - `&`, if will be checked by number of segments seperated by `&`
                let input_vector: Vec<&str> = input.split("&").collect();
                if input_vector.len() != 3 {
                    println!("Illegal input. Input should not contain `&` character");
                    std::process::exit(1);              
                }
                let block_hash = correct_url(input_vector[0]);
                let address = correct_url(input_vector[1]);
                let key = correct_url(input_vector[2]);

                let route = format!("{}/worldstate?block_hash={}&address={}&key={}", route_address, block_hash, address, key);
                query_helper(route, QueryReturnType::Text, client).await;
            }
        },
        QueryOption::GasPerBlock |  QueryOption::MempoolSize => {
            if let Some(input) = query_input {
                // We use `&` character as delimiter of input vector as all field are checked to be integer
                let input_vector: Vec<&str> = input.split("&").collect();
                if input_vector.len() != 4 {
                    println!("Illegal input. Input should not contain `&` character");
                    std::process::exit(1);              
                }
                let from_time:u64 = match input_vector[0].parse(){
                    Ok(value) => value,
                    Err(_) => {
                        println!("Illegal input. From time should be a number");
                        std::process::exit(1); 
                    }
                };
                let to_time:u64 = match input_vector[1].parse(){
                    Ok(value) => value,
                    Err(_) => {
                        println!("Illegal input. To time should be a number");
                        std::process::exit(1); 
                    }
                };
                let window_size:u64 =  match input_vector[2].parse(){
                    Ok(value) => value,
                    Err(_) => {
                        println!("Illegal input. Window size should be a number");
                        std::process::exit(1); 
                    }
                };
                let step_size:u64 =  match input_vector[3].parse(){
                    Ok(value) => value,
                    Err(_) => {
                        println!("Illegal input. Step size should be a number");
                        std::process::exit(1); 
                    }
                };
                let category = match query_option {
                    QueryOption::GasPerBlock => "gasperblock",
                    QueryOption::MempoolSize => "mempoolsize",
                    _ => unreachable!()
                };

                let route = format!("{}/analytics?category={}&from_time={}&to_time={}&window_size={}&step_size={}", route_address, category, from_time, to_time, window_size, step_size);
                query_helper(route, QueryReturnType::VectorU64, client).await;
            }
        },
        QueryOption::None => {println!("You should not reach here.")},
    }
}

// Correct the base64 encoding string by replacing slash character with an equivalent "%2F" character and plus character with "%2B"
// There is currently no need for other percentage encoding as base64 encoding only contains this two `special` character
pub(crate) fn correct_url(string_to_url: &str) -> String{
    string_to_url.replace("/", "%2F")
                 .replace("+", "%2B")
}

enum QueryReturnType{
    Block,
    Blocks,
    Transaction,
    VectorU64,
    Text,
}

// Query helper sned the request and helps to transalte the return result from restAPI endpoints to more human readable content.
// e.g. Bock and Transaction which is in protobuf bytes will be formatted and beautified.
// e.g. Analytics result which should be in vector of u64 return comes in vector of u8 when return from API. We also had to transalte it back. 
async fn query_helper(route: String, query_return_type:QueryReturnType, client: reqwest::Client){
    let resp = match client.get(route)
    .send()
    .await {
        Ok(some_resp) => some_resp,
        Err(e) => {
            println!("Error: Server connection error");
            println!("Detail: {}", format!("{}", e));
            std::process::exit(1);
        }
    };

    if resp.status().is_success() {
        match query_return_type {
            QueryReturnType::Block => {
                let block_return = protocol_types::Block::deserialize(&resp.bytes().await.unwrap()).unwrap();
                let block_print: Block = From::<protocol_types::block::Block>::from(block_return);
                println!("Your Block: {:#?}", block_print);
            },
            QueryReturnType::Blocks => {
                let blocks_return = protocol_types::Blocks::deserialize(&resp.bytes().await.unwrap()).unwrap();
                let blocks_print: Blocks = From::<protocol_types::block::Blocks>::from(blocks_return);
                println!("Your Blocks: {:#?}", blocks_print);
            },
            QueryReturnType::Transaction => {
                let tx_return = protocol_types::Transaction::deserialize(&resp.bytes().await.unwrap()).unwrap();
                let tx_print: Transaction = From::<protocol_types::transaction::Transaction>::from(tx_return);
                println!("Your Tx {:#?}", tx_print);
            },
            QueryReturnType::VectorU64 => {
                let value = &resp.bytes().await.unwrap().to_vec();
                let u64vec_from_u8bytes:Vec<u64> = (0..value.len()/8).map(
                    |index|{
                        u64::from_be_bytes(value[8*index..8*(index+1)].try_into().unwrap())
                    }
                ).collect();
                println!("Your value {:?}", u64vec_from_u8bytes);
            },
            QueryReturnType::Text => {
                let value = &resp.text().await.unwrap();
                let decoded_value = match base64::decode(value){
                    Ok(decode) => decode,
                    Err(_) => {
                        println!("Your value is: {:?}", value);
                        std::process::exit(1);
                    }
                };
                println!("Your value {:?}", value);
                println!("Your value(decoded) {:?}", decoded_value);
            }
        }
    }else {
        println!("Query Error. Status: {:?}", resp.status());
        let resp_detail = resp.text().await.unwrap();
        println!("Query Error. Detail: {}", http_formatted(resp_detail));
    } 
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