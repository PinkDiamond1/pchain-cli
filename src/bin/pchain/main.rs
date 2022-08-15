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
pub mod operations;
pub mod display_types;
pub mod args_parser;
pub mod prt_parser;

use clap::{Parser, Subcommand};
use operations::{submit, query, setup::{self, *}, QueryOption, SubmitTxJson};


type Base64Address = String;
type Base64Hash = String;
// type Base64Keypair = String;
type Base64String = String;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A simple CLI to submit Transactions and query data from the ParallelChain Mainnet.  
#[derive(Debug, Parser)]
#[clap(name = format!("ParallelChain F 'VeryLight' Client v{}", VERSION))]
#[clap(about = "VeryLight is an easy-to-use CLI for interacting with ParallelChain F (Mainnet) networks. If you're new, start by setting up VeryLight using the 'Setup' command.", author = "<ParallelChain Lab>", long_about = None)]
enum VeryLightCLI {
    /// Set up configuration variables necessary for VeryLight's operation.
    #[clap(display_order=1)]
    Setup {
        #[clap(subcommand)]
        set_subcommand: Setup,
    },

    /// Construct and submit Transactions up to the ParallelChain Mainnet network.
    #[clap(display_order=2)]
    Submit {
        #[clap(subcommand)]
        submit_subcommand: Submit,
    },

    /// Query the ParallelChain Mainnet network for protocol-defined constructs like Accounts, Blocks, World State, etc. 
    #[clap(display_order=3)]
    Query {
        #[clap(subcommand)]
        query_subcommand: Query,
    },

    #[clap(display_order=4)]
    Crypto {
        #[clap(subcommand)]
        crypto_subcommand: Crypto,
    },

    /// Query analytical information about the state of the ParallelChain Mainnet network like Gas Per Block, and Mempool Size.
    #[clap(display_order=5)]
    Analyze {
        #[clap(subcommand)]
        analyze_subcommand: Analyze,
    },

    /// [Experimental] Parser Tool for creating data argument from simpla basic rust data structures
    #[clap(display_order=6)]
    Parse {
        #[clap(subcommand)]
        parse_subcommand: Parse,
    }
}

#[derive(Debug, Subcommand)]
enum Submit {
    /// Submit tx
    #[clap(arg_required_else_help = true, display_order=1)]
    Tx {
        /// 'Sending address' of this Transaction. Base64 encoded Ed25519 Public Key (32 bytes).
        #[clap(long="from-address", display_order=1)]
        from_address : Base64Address,

        /// 'Receiver address' of this Transaction. Can be either: 1. a Base64 encoded Ed25519 Public Key (32 bytes) identifying an External Account or a Contract Account, or 2., 'null', if this is a Deploy Transaction.
        #[clap(long="to-address", display_order=2)]
        to_address : Base64Address,

        /// XPLL/TXPLL to transfer to the account identified by to-address (in Grays).
        #[clap(long="value", display_order=3)]
        value: u64,

        /// XPLL/TXPLL to tip to the proposing Validator. Set this to a high value if you need Transaction to be included in a block quickly (in Grays).
        #[clap(long="tip", display_order=4)]
        tip: u64,

        /// Maximum number of Gas units that you are willing to consume on executing this Transaction. If this is set to low, your Transaction may not execute to completion.
        #[clap(long="gas-limit", display_order=5)]
        gas_limit: u64,

        /// XPLL/TXPLL you are willing to pay per unit Gas consumed in the execution of your transaction (in Grays). This needs to be greater than your Account balance for your transaction to be included in a block. 
        #[clap(long="gas-price", display_order=6)]
        gas_price: u64,

        /// Free field. Can be either 1. 'null', or 2., a Base64 encoded message that will be included in the Blockchain and passed into the Smart Contract
        /// (if 'to-address' identifies a Contract Account), or 3. A relative path (starting with '.') to a Compiled Smart Contract '.wasm' file, if this is a Deploy Transaction.
        #[clap(long="data", display_order=7)]
        data: String,

        /// [Optional] Base64-encoded arguments for contract deployment. These are passed into the "init" entrypoint of the deployed Contract.
        #[clap(long="deploy-args", display_order=8)]
        deploy_args: Option<String>,

        /// Number of Transactions included on-Chain from 'from_address', or 'nonce', for short. You can simply use the nonce you get by Query-ing on from_address directly.
        #[clap(long="nonce", display_order=9)]
        nonce: u64,

        /// Relative path to a JSON file containing your secret key, public key, and keypair. Read the VeryLight repository README.md for the file format, or generate a 'keypair.json' using the 'Setup' command.
        /// This is used to produce a cryptographic signature that proves that 'you' are authorized to make this Transaction.
        #[clap(long="path-to-keypair-json", display_order=10)]
        keypair: String,
    },

    /// Submit tx from json file
    #[clap(arg_required_else_help = true, display_order=2)]
    TxFrom {

        /// Relative path to a JSON file of Transaction. Example json file:
        /// {
        ///   "from_address": "1a99UDMoXm88AdzeGSmeOQOX0NHpMRcnTW1IcE7Nwl4=",
        ///   "to_address": "MJrfQCg_7Gb7Spw6v8zalYIETwwks8aoI7HrGofMRHY",
        ///   "value": 1,
        ///   "tip": 0,
        ////   "gas_limit": 67500000,
        ///   "gas_price": 1,
        ///   "data": "",
        ///   "deploy_args": "",
        ///   "nonce": 138,
        ///   "path_to_keypair_json": "../keypair.json"
        /// }
        #[clap(long="file", display_order=1)]
        file: String,
    }
}


#[derive(Debug, Subcommand)]
enum Query {
    /// Query information related to Accounts.
    #[clap(arg_required_else_help = true, display_order=1)]
    Account {
        #[clap(subcommand)]
        account_subcommand: Account,
    },

    /// Query information related to Blocks. Search the blocks either by block number, block hash or tx hash.
    #[clap(arg_required_else_help = true, display_order=2)]
    Blocks {
        /// Block number (a.k.a., Block 'height') of the Block you'd like to query.
        #[clap(long="block-num", display_order=1)]
        block_num : Option<u64>,

        /// Block hash of the Block you'd like to query.
        #[clap(long="block-hash", display_order=2)]
        block_hash : Option<Base64Hash>,

        /// Hash of the Transaction you'd like to query the containing Block of.
        #[clap(long="tx-hash", display_order=3)]
        tx_hash : Option<Base64Hash>,

        /// Specify this flag to query from the latest block
        #[clap(long="latest", display_order=4)]
        latest : bool,

	    /// Size of query window.
        #[clap(long="size", display_order=5)]
	    size: u64,

        /// "true" or "false". Specifying the former causes this endpoint to return only BlockHeaders. Specifying the latter causes the endpoint to also return Blocks' Transactions (sans Receipts and Events)
        #[clap(long="header-only", display_order=6)]
        header_only: String,

        /// "true", "false". Default to be "false". Specifying the former causes this endpoint to return a summary of header only. Covering height, block_hash, state_hash, receipts_hash, time, tx_count( if header_only is false)
        #[clap(long="summary-only", display_order=7)]
        summary_only: Option<String>
    },

    /// Query VeryLight's network configuration 
    #[clap(arg_required_else_help = true, display_order=3)]
    Networking {
        /// Target URL (Standard API) of ParallelChain F (Mainnet).
        #[clap(long="target-url", display_order=1)]
        target_url : bool,

        /// Rich API URL of ParallelChain F (Mainnet).
        #[clap(long="rich-api-url", display_order=2)]
        rich_api_url : bool,

        /// Analytics URL of ParallelChain F (Mainnet).
        #[clap(long="analytics-api-url", display_order=3)]
        analytics_api_url : bool,
    },

    /// Query Keys in the World State of Contract Accounts.
    #[clap(arg_required_else_help = true, display_order=4)]
    State {
	    /// Address of interested contract
        #[clap(long="address", display_order=1)]
	    address: Base64Address,

	    /// Key of world state. BASE64 encoded of key defined in contract
        #[clap(long="key", display_order=2)]
	    key: Base64String,
    },

    /// Query multiple Transactions.
    #[clap(arg_required_else_help = true, display_order=5)]
    Txs {
        /// Transaction number (a.k.a., Transaction 'height') of the Transaction you'd like to query.
        #[clap(long="tx-num", display_order=1)]
        tx_num : Option<u64>,

        /// Block hash of the Transaction you'd like to query.
        #[clap(long="tx-hash", display_order=2)]
        tx_hash : Option<Base64Hash>,

        /// Specify this flag to query from the latest transaction
        #[clap(long="latest", display_order=3)]
        latest : bool,

        /// Size of query window.
        #[clap(long="size", display_order=4)]
	    size: u64,

        /// "true", "false". Default to be "false". Specifying the former causes this endpoint to return a summary of header only. Covering height, block_hash, state_hash, receipts_hash, time, tx_count( if header_only is false)
        #[clap(long="summary-only", display_order=5)]
        summary_only: Option<String>
    },

    /// Query Transaction Proof
    #[clap(arg_required_else_help = true, display_order=6)]
    TxProof {
	    /// Identifies the target Block
        #[clap(long="block-hash", display_order=1)]
        block_hash: Base64Hash,
	    /// Identifies the target Transaction
        #[clap(long="tx-hash", display_order=2)]
        tx_hash: Base64Hash,
    },
    /// Query Receipt Proof
    #[clap(arg_required_else_help = true, display_order=7)]
    ReceiptProof {
	    /// Identifies the target Block
        #[clap(long="block-hash", display_order=1)]
        block_hash: Base64Hash,
	    /// Identifies the target Transaction
        #[clap(long="tx-hash", display_order=2)]
        tx_hash: Base64Hash,
    },

    /// Query size of mempool
    #[clap(arg_required_else_help = false, display_order=8)]
    Mempoolsize
}

#[derive(Debug, Subcommand)]
enum Setup {
    /// Configure the IP address of the ParallelChain F Fullnode that VeryLight sends Transactions and queries to. 
    #[clap(arg_required_else_help = true, display_order=1)]
    Networking {
        /// Configure the URL of the ParallelChain F Fullnode that VeryLight sends Transactions and queries to. (Standard API) 
        #[clap(long="target-url", required = false,  display_order=1)]
        target_url: Option<String>,

        /// Configure the URL of the ParallelChain F Fullnode that VeryLight sends rich queries. (Rich API) 
        #[clap(long="rich-api-url", required = false, display_order=2)]
        rich_api_url: Option<String>,

        /// Configure the URL of the ParallelChain F Fullnode that VeryLight sends analytics queries (Analytics API) 
        #[clap(long="analytics-api-url", required = false, display_order=3)]
        analytics_api_url: Option<String>,

    },

    /// Register an Ed25519 KeyPair for VeryLight to use to sign Transactions.
    #[clap(arg_required_else_help = true, display_order=2)]
    KeyPair {
        /// Absolute path to a 'keypair.json' file. You can generate this file using the Crypto command. Read VeryLight's repository README for the file format. 
        #[clap(long="keypair-json-path", display_order=1)]
        keypair_json_path: String,
    },
}

#[derive(Debug, Subcommand)]
enum Crypto {
     /// Generate a Ed25519 and save the key material in a 'keypair.json' file.
    #[clap(arg_required_else_help = false, display_order=1)]
    GenerateKeyPair,

    /// Sign a message using VeryLight's registered KeyPair. (If you've not registered a KeyPair, use the Setup command).
    /// This prints out the resulting ciphertext in Base64 encoding.
    #[clap(arg_required_else_help = true, display_order=2)] 
    Sign {
        /// A message to sign, encoded in Base64.
        #[clap(long="message", display_order=1)]
        message: String,
    } 
}

#[derive(Debug, Subcommand)]
enum Parse {

    /// Parse data in json file into arguments to contract method
    #[clap(arg_required_else_help = true, display_order=1)]
    Calldata {
        
        /// Relative Path to json file
        /// Accept data format: i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, bool, String, [32], [64],
        /// Vec<i8>, Vec<i16>, Vec<i32>, Vec<i64>, Vec<i128>, Vec<u8>, Vec<u16>, Vec<u32>, Vec<u64>, Vec<u128>, 
        /// Vec<bool>, Vec<String>, address.
        /// Example values in Vec or slice: [0,1,2].
        /// The data type [32] and [64] refers to slice of 32 bytes and slice of 64 bytes. 
        /// `address` must be base64url encoded string
        #[clap(long="json-file", display_order=1)]
        json_file :String

    },

    /// Parse return value from result of contract call.
    #[clap(arg_required_else_help = true, display_order=2)]
    Callback {

        /// The returned base64 string from result of contract call.
        #[clap(long="value", display_order=1)]
        value: String,

        /// Accept data type: i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, bool, String, [32], [64],
        /// Vec<i8>, Vec<i16>, Vec<i32>, Vec<i64>, Vec<i128>, Vec<u8>, Vec<u16>, Vec<u32>, Vec<u64>, Vec<u128>, 
        /// Vec<bool>, Vec<String>. 
        /// Example values in Vec or slice: [0,1,2].
        /// The data type [32] and [64] refers to slice of 32 bytes and slice of 64 bytes. 
        #[clap(long="data-type", display_order=2)]
        data_type: String,
    },

    /// Parse protocol types file to display the data in the structure
    #[clap(arg_required_else_help = true, display_order=3)]
    Prt {
        #[clap(long="file", display_order=1)]
        file: String
    }
}

#[derive(Debug, Subcommand)]
enum Analyze {
    /// Gas-Per-Block computed on the basis of a moving-window average.
    #[clap(arg_required_else_help = true, display_order=1)]
    GasPerBlock {
        /// Start-time of period of interest (Unix timestamp).
        #[clap(long="start-time", display_order=1)]
        start_time: u64,

        /// End-time of period of interest (Unix timestamp).
        #[clap(long="end-time", display_order=2)]
        end_time: u64,

        /// Moving window size (in seconds).
        #[clap(long="window-size", display_order=3)]
        window_size: u64,

        /// Moving window step-size in seconds. E.g., Non overlapping moving windows have step-size == window size.
        #[clap(long="step-size", display_order=4)]
        step_size: u64,
    },

    /// Mempool size (in bytes) computed on the basis of a moving-window average.
    #[clap(arg_required_else_help = true, display_order=2)]
    MempoolSize {
        /// Start-time of period of interest (Unix timestamp).
        #[clap(long="start-time", display_order=1)]
        start_time: u64,

        /// End-time of period of interest (Unix timestamp).
        #[clap(long="end-time", display_order=2)]
        end_time: u64,

        /// Moving window size (in seconds). 
        #[clap(long="window-size", display_order=3)]
        window_size: u64,

        /// Moving window step size in seconds. E.g., Non overlapping moving window should have step size = window size 
        #[clap(long="step-size", display_order=4)]
        step_size: u64
    },
}

#[derive(Debug, Subcommand)]
enum Account {
    /// Query an Account's balance (in Grays).
    #[clap(arg_required_else_help = true, display_order=1)]
    Balance {
        /// Address of the External or Contract Account you'd like to query.
        #[clap(long="address", display_order=1)]
        address: Base64Address,
    },

    /// Query the number of Transactions originating from an External Account that has been included on Chain (a.k.a., the nonce).
    #[clap(arg_required_else_help = true, display_order=2)]
    Nonce {
        /// Address of the External Account you'd like to query.
        #[clap(long="address", display_order=1)]
        address: Base64Address,
    },

    /// Query a Contract account's Contract Byte Code (Base64 encoded). 
    #[clap(arg_required_else_help = true, display_order=3)]
    ContractCode {
        /// Address of the External Account you'd like to query. 
        #[clap(long="address", display_order=1)]
        address: Base64Address,
    },

    /// Query a Contract account's Contract Metadata (can be null).
    #[clap(arg_required_else_help = true, display_order=4)]
    ContractMetadata {
        /// Address of the Contract Account you'd like to query.
        #[clap(long="address", display_order=1)]
        address: Base64Address,
    },
    /// Query contract by accessing the view entrypoint method.
    #[clap(arg_required_else_help = true, display_order=5)]
    View {
        /// Address of the Contract Account you'd like to query.
        #[clap(long="address", display_order=1)]
        address: Base64Address,

        /// Arguments to view entrypoint method.
        #[clap(long="calldata", display_order=2)]
        calldata: Base64String,

        /// Expected return data type from contract view method. Leave blank if not use.
        /// Accept inputs: i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, bool, String, [32], [64],
        /// Vec<i8>, Vec<i16>, Vec<i32>, Vec<i64>, Vec<i128>, Vec<u8>, Vec<u16>, Vec<u32>, Vec<u64>, Vec<u128>, 
        /// Vec<bool>, Vec<String>. 
        #[clap(long="expected", display_order=3)]
        expected_return_type: Option<String>
    }
}

// This maps the argument collection to the corresponding handling function
#[tokio::main]
async fn main() {
    // This is the argument collector
    let args = VeryLightCLI::parse();

    // This maps the argument collection to the corresponding function
    match args {
        VeryLightCLI::Submit { submit_subcommand } => {
            match submit_subcommand {
                Submit::Tx { from_address, to_address, value, tip, gas_limit, gas_price, mut data, deploy_args, nonce, keypair } => {
                    if data.to_lowercase() == "null" { data = "".to_string() };
                    let deploy_args = match deploy_args { Some(str) => str, None=> "".to_string() };
                    let is_deploy = &to_address == "null"; // To address is null if and only if it is a deploy transaction
                    let submit_tx_json = SubmitTxJson { 
                        from_address, 
                        to_address, 
                        value, 
                        tip, 
                        gas_limit, 
                        gas_price, 
                        data, 
                        deploy_args, 
                        nonce, 
                        path_to_keypair_json: keypair
                    };
                    submit(submit_tx_json, is_deploy).await
                },
                Submit::TxFrom { file } => {
                    let tx_json = SubmitTxJson::load_tx_json_file(file);
                    let is_deploy = &tx_json.to_address == "null"; // To address is null if and only if it is a deploy transaction
                    submit(tx_json, is_deploy).await
                }
            }
        },

        VeryLightCLI::Query { query_subcommand } => {
            match query_subcommand {
                Query::Account{ account_subcommand } => {
                    match account_subcommand {
                        Account::Balance{address} => {
                            query(QueryOption::Balance, vec![address]).await;
                        },
                        Account::Nonce{address} => {
                            query(QueryOption::Nonce, vec![address]).await;
                        },
                        Account::ContractCode{address} => {
                            query(QueryOption::ContractCode, vec![address]).await;
                        },
                        Account::ContractMetadata {address} => {
                            query(QueryOption::ContractMetadata, vec![address]).await;
                        },
                        Account::View { address, calldata, expected_return_type } => {
                            let expected_return_type = match  expected_return_type {
                                Some(s) => s,
                                None => "".to_string()
                            };
                            query(QueryOption::View, vec![address, calldata, expected_return_type]).await;
                        }
                    }
                }
                Query::Blocks { block_num, block_hash, tx_hash, latest, size, header_only, summary_only } => {
                    let summary_only = match summary_only {
                        Some(s) => s,
                        None => "".to_string()
                    };
                    if latest {
                        query(QueryOption::BlocksLatest, vec!["true".to_string(), size.to_string(), header_only, summary_only]).await;    
                    } else if let Some(num) = block_num {
                        query(QueryOption::BlocksByBlockNum, vec![num.to_string(), size.to_string(), header_only, summary_only]).await;
                    } else if let Some(hash) = block_hash {
                        query(QueryOption::BlocksByBlockHash, vec![hash, size.to_string(), header_only, summary_only]).await;
                    } else if let Some(hash) =  tx_hash {
                        query(QueryOption::BlocksByTxHash, vec![hash, size.to_string(), header_only, summary_only]).await;
                    }
                },
                Query::Networking { target_url, rich_api_url, analytics_api_url } => {
                    if target_url == true { println!("target_url is {}", setup::read_config(ConfigField::TargetUrl)) }
                    if rich_api_url == true { println!("rich_api_url is {}", setup::read_config(ConfigField::RichApiUrl)) }
                    if analytics_api_url == true { println!("analytics_api_url is {}", setup::read_config(ConfigField::AnalyticsApiUrl)) }
                },
                Query::State { address, key } => {
                    query(QueryOption::WorldState, vec![address, key]).await;
                },
                Query::Txs { tx_num, tx_hash, size, latest, summary_only } => {
                    let summary_only = match summary_only {
                        Some(s) => s,
                        None => "".to_string()
                    };
                    if latest {
                        query(QueryOption::TxsLatest, vec!["true".to_string(), size.to_string(), summary_only]).await;
                    } else if let Some(tx_num) = tx_num {
                        query(QueryOption::TxsByTxNum, vec![tx_num.to_string(), size.to_string(), summary_only]).await;
                    } else if let Some(tx_hash) = tx_hash {
                        query(QueryOption::TxsByTxHash, vec![tx_hash.to_string(), size.to_string(), summary_only]).await;
                    }
                }
                Query::TxProof { block_hash, tx_hash } => {
                    query(QueryOption::TxProof, vec![block_hash, tx_hash]).await;
                },
                Query::ReceiptProof { block_hash, tx_hash} => {
                    query(QueryOption::ReceiptProof, vec![block_hash, tx_hash]).await;
                },
                Query::Mempoolsize => {
                    query(QueryOption::Mempoolsize, vec![]).await;
                }
            }
        },

        VeryLightCLI::Setup { set_subcommand } => {
            match set_subcommand {
                Setup::Networking { target_url, rich_api_url, analytics_api_url } => {
                    // setup each networking key based on the value the user inputs.
                    match target_url {
                        Some(t) => setup::set_config(ConfigField::TargetUrl ,&t),
                        None => (),
                    };
                    match rich_api_url {
                        Some(r) => setup::set_config(ConfigField::RichApiUrl ,&r),
                        None => (),
                    };
                    match analytics_api_url {
                        Some(a) => setup::set_config(ConfigField::AnalyticsApiUrl ,&a),
                        None => (),
                    };
     
                },
                Setup::KeyPair { keypair_json_path } => {
                    setup::set_config(ConfigField::KeypairJSONPath, &keypair_json_path);
                },
            }
        },

        VeryLightCLI::Crypto { crypto_subcommand } => {
            match crypto_subcommand {
                Crypto::GenerateKeyPair => {
                    operations::crypto::generate_keypair_and_save_as_json();
                }

                Crypto::Sign { message } => {
                    operations::crypto::sign(&message);
                }
            }
        }

        VeryLightCLI::Analyze { analyze_subcommand } => {
            match analyze_subcommand {
                Analyze::GasPerBlock {  start_time, end_time, window_size, step_size } => {
                    query(QueryOption::GasPerBlock, vec![
                        start_time.to_string(), 
                        end_time.to_string(), 
                        window_size.to_string(), 
                        step_size.to_string()
                    ]).await;                
                },
                Analyze::MempoolSize { start_time, end_time, window_size, step_size} => {
                    query(QueryOption::MempoolSize, vec![
                        start_time.to_string(), 
                        end_time.to_string(), 
                        window_size.to_string(), 
                        step_size.to_string()
                    ]).await;
                },
            }          
        },

        VeryLightCLI::Parse { parse_subcommand } => {
            match parse_subcommand {
                Parse::Calldata { json_file } => {
                    let (output_data_str,_) = args_parser::parse(json_file);
                    println!("Note: Base64 encoded output string for `data` can be used in command `submit tx` and `query account view`.");
                    println!("\n{}\n", output_data_str);
                },
                Parse::Callback { value, data_type } => {
                    let result = args_parser::from_callback(value, data_type);
                    println!("{}", result);
                },
                Parse::Prt { file } => {
                    let output = prt_parser::parse_file(file);
                    println!("{}", output);
                }
            }
        }
    };
}
