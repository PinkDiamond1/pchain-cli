pub mod operations;
pub mod display_types;

use clap::{Parser, Subcommand};
use operations::{submit, query, setup::{self, *}, QueryOption };

type Base64Address = String;
type Base64Hash = String;
type Base64Keypair = String;
type Base64String = String;

/// A simple CLI to submit Transactions and query data from the ParallelChain Mainnet.  
#[derive(Debug, Parser)]
#[clap(name = "ParallelChain F 'VeryLight' Client")]
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
        #[clap(long="path_to_keypair_json", display_order=10)]
        keypair: String,
    },
}


#[derive(Debug, Subcommand)]
enum Query {
    /// Query information related to Accounts.
    #[clap(arg_required_else_help = true, display_order=1)]
    Account {
        #[clap(subcommand)]
        account_subcommand: Account,
    },

    /// Query information related to Blocks.
    #[clap(arg_required_else_help = true, display_order=2)]
    Block {
        /// Block number (a.k.a., Block 'height') of the Block you'd like to query.
        #[clap(long="block-num", display_order=1)]
            block_num : Option<u64>,

        /// Block hash of the Block you'd like to query.
        #[clap(long="block-hash", display_order=2)]
            block_hash : Option<Base64Hash>,

        /// Hash of the Transaction you'd like to query the containing Block of.
        #[clap(long="tx-hash", display_order=3)]
            tx_hash : Option<Base64Hash>,

        /// Get latest Block.
        #[clap(long="latest", display_order=4)]
            latest : bool,
    },

    /// Query the Block number of an Block by its hash.
    #[clap(arg_required_else_help = true, display_order=3)]
    BlockNum {
        #[clap(long="block-hash", display_order=1)]
	    block_hash: Base64Hash,
    },

    /// Query VeryLight's target address.
    #[clap(arg_required_else_help = true, display_order=4)]
    Config {
        #[clap(long="target-address", display_order=1)]
    	target_address: bool,
    },

    /// Query information related to multiple consecutive Blocks (up to 100).
    #[clap(arg_required_else_help = true, display_order=5)]
    MultiBlocks {
	    /// Identifies last Block in query window.
        #[clap(long="block-hash", display_order=1)]
	    block_hash: Base64Hash,

	    /// Size of query window.
        #[clap(long="size", display_order=2)]
	    size: u64,
    },

    /// Query Keys in the World State of Contract Accounts.
    #[clap(arg_required_else_help = true, display_order=6)]
    State {
	    /// Snapshot time of world state - represneted by block hash
        #[clap(long="block-hash", display_order=1)]
	    block_hash: Base64Hash,

	    /// Address of interested contract
        #[clap(long="size", display_order=2)]
	    address: Base64Address,

	    /// Key of world state. BASE64 encoded of key defined in contract
        #[clap(long="key", display_order=3)]
	    key: Base64String,
    },

    /// Query keys related to Transactions.
    #[clap(arg_required_else_help = true, display_order=7)]
    Tx {
        /// Hash of the Transaction you'd like to query. 
        #[clap(long="tx-hash", display_order=1)]
        tx_hash : Base64Hash,
    },
}

#[derive(Debug, Subcommand)]
enum Setup {
    /// Configure the IP address of the ParallelChain F Fullnode that VeryLight sends Transactions and queries to. 
    #[clap(arg_required_else_help = true, display_order=1)]
    Networking {
        /// Configure the IP address of the ParallelChain F Fullnode that VeryLight sends Transactions and queries to. 
        #[clap(long="target-address", display_order=1)]
        target_address: String,
    },

    /// Register an Ed25519 KeyPair for VeryLight to use to sign Transactions.
    #[clap(arg_required_else_help = true, display_order=2)]
    KeyPair {
        /// Absolute path to a 'keypair.json' file. You can generate this file using the Crypto command. Read VeryLight's repository README for the file format. 
        #[clap(long="keypair_json_path", display_order=1)]
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
                    submit(from_address, to_address, value, tip, gas_limit, gas_price,data, deploy_args, nonce, keypair).await
                }
            }
        },

        VeryLightCLI::Query { query_subcommand } => {
            match query_subcommand {
                Query::Account{ account_subcommand } => {
                    match account_subcommand {
                        Account::Balance{address} => {
                            query(QueryOption::Balance, Some(&address)).await;
                        },
                        Account::Nonce{address} => {
                            query(QueryOption::Nonce, Some(&address)).await;
                        },
                        Account::ContractCode{address} => {
                            query(QueryOption::ContractCode, Some(&address)).await;
                        },
                        Account::ContractMetadata {address} => {
                            query(QueryOption::ContractMetadata, Some(&address)).await;
                        },
                    }
                }
                Query::Block { block_num, block_hash, tx_hash, latest } => {
                    if let Some(num) = block_num {
                        query(QueryOption::BlockByBlockNum, Some(&num.to_string())).await;
                    } 
                    else if let Some(hash) = block_hash {
                        query(QueryOption::BlockByBlockHash, Some(&hash)).await;
                    }
                    else if let Some(hash) =  tx_hash {
                        query(QueryOption::BlockByTxHash, Some(&hash)).await;
                    } 
                    else if latest == true {
                        query(QueryOption::LatestBlock, None).await;
                    }
                },
                Query::BlockNum { block_hash } => {
                    query(QueryOption::BlockNumByBlockHash, Some(&block_hash)).await;
                },
                Query::Config { target_address: _ } => {
                    println!("target_address is {}", setup::read_config(ConfigField::TargetAddress))
                },
                Query::MultiBlocks { block_hash, size } => {
                    let query_input = format!("{}&{}", block_hash, size);
                    query(QueryOption::BlocksByBlockHash, Some(&query_input)).await;
                },
                Query::State { block_hash, address, key } => {
                    // & is used as a seperator
                    let state_query_str = format!("{}&{}&{}", block_hash, address, key);
                    query(QueryOption::WorldState, Some(&state_query_str)).await;
                },
                Query::Tx { tx_hash } => {
                    query(QueryOption::TxByTxHash, Some(&tx_hash)).await;
                },
            }
        },

        VeryLightCLI::Setup { set_subcommand } => {
            match set_subcommand {
                Setup::Networking { target_address } => {
                    setup::set_config(ConfigField::TargetAddress ,&target_address)
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
                    let analytics_query_str = format!("{}&{}&{}&{}", start_time, end_time, window_size, step_size);
                    query(QueryOption::GasPerBlock, Some(&analytics_query_str)).await;                
                },
                Analyze::MempoolSize { start_time, end_time, window_size, step_size} => {
                    let analytics_query_str = format!("{}&{}&{}&{}", start_time, end_time, window_size, step_size);
                    query(QueryOption::MempoolSize, Some(&analytics_query_str)).await;
                },
            }          
        },
    };
}
