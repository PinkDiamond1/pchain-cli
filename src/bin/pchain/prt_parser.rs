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
use crate::display_types;

use protocol_types::Deserializable;
use std::path::Path;



pub fn parse_file(path_to_ptr: String) -> String {

    let names: Vec<&str> = path_to_ptr.split(".").collect();
    if names.len() < 3 {
        println!("Error: : Invalid file name. File extension must be .<data type extension>.prt");
        std::process::exit(1);
    }
    if names[names.len()-1] != "prt" {
        println!("Error: : Invalid file extension. File extension must be .<data type extension>.prt");
        std::process::exit(1);
    }

    let ext = names[names.len()-2];
    
    if !is_correct_data_type_ext(ext) {
        println!("Error: : Not recognized data type extension");
        std::process::exit(1);
    };

    let file_binary = if Path::new(&path_to_ptr).is_file(){
        match std::fs::read(&path_to_ptr) {
            Ok(data) => data,
            Err(e) => {
                println!("Error: : Fail to read file although is file found {:?}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("Error: : Invalid path. Cannot retrieve Prt file from the designated path.");
        std::process::exit(1);
    };

    to_display_types(&file_binary, ext)
}

fn is_correct_data_type_ext(ext: &str) -> bool {
    ["bh", "blk", "tx", "recp", "evt", "call", "mprf", "sprfs"].contains(&ext)
}

fn to_display_types(serialized_data: &Vec<u8>, ext: &str) -> String {
    macro_rules! from_prt_to_return_display_types {
        ($ext:expr, $($e:expr, $m1:tt::$t1:tt => $m2:tt::$t2:tt)*) => {
            match $ext {
                $(
                    $e => {
                        if let Ok(value) = $m1::$t1::deserialize(&serialized_data) {
                            return format!("{:?}", $m2::$t2::from(value));
                        }
                    }
                )*

                _=> {}
            }
            
        };
    }

    from_prt_to_return_display_types!(ext,
        "bh", protocol_types::BlockHeader => display_types::BlockHeader
        "blk", protocol_types::Block => display_types::Block
        "tx", protocol_types::Transaction => display_types::Transaction
        "recp", protocol_types::Receipt => display_types::Receipt
        "evt", protocol_types::Event => display_types::Event
        "call", protocol_types::CallData => display_types::CallData
        "mprf", protocol_types::MerkleProof => display_types::MerkleProof
        "sprfs", protocol_types::StateProofs => display_types::StateProofs
    );
    
    "".to_string()
}


#[cfg(test)]
mod test {
    use protocol_types::Serializable;

    #[test]
    fn test_to_display_types() {
        let block_header = protocol_types::BlockHeader {
            blockchain_id :9828192,
            block_version_number : 2,
            block_number: 1,
            timestamp : 3,
            prev_block_hash : [1u8; 32],
            this_block_hash : [2u8; 32],
            txs_hash : [1u8,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2],
            state_hash : [4u8; 32],
            receipts_hash : [6u8; 32],
            proposer_public_key : [7u8; 32],
            signature : [8u8; 64],
        };
        let serialized_bh = protocol_types::BlockHeader::serialize(&block_header);
        let output = super::to_display_types(&serialized_bh, "bh");
        let result = r#"BlockHeader { blockchain_id: 9828192, block_version_number: 2, block_number: 1, timestamp: 3, prev_block_hash: "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE", this_block_hash: "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI", txs_hash: "AQIDBAUGBwgJAAECAwQFBgcICQABAgMEBQYHCAkAAQI", state_hash: "BAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQ", receipts_hash: "BgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgY", proposer_public_key: "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc", signature: "CAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICA" }"#;
        assert_eq!(output, result);
      
        let block = protocol_types::Block {
            // add data
            header: protocol_types::BlockHeader {
                    blockchain_id :9828192,
                    block_version_number : 2,
                    block_number: 1,
                    timestamp : 3,
                    prev_block_hash : [1u8; 32],
                    this_block_hash : [2u8; 32],
                    txs_hash : [1u8,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2],
                    state_hash : [4u8; 32],
                    receipts_hash : [6u8; 32],
                    proposer_public_key : [7u8; 32],
                    signature : [8u8; 64],
            },
            transactions: generate_transactions(),
            receipts: generate_receipts(),
        };
        let serialized_blk = protocol_types::Block::serialize(&block);
        let output = super::to_display_types(&serialized_blk, "blk");
        let result = r#"Block { header: BlockHeader { blockchain_id: 9828192, block_version_number: 2, block_number: 1, timestamp: 3, prev_block_hash: "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE", this_block_hash: "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI", txs_hash: "AQIDBAUGBwgJAAECAwQFBgcICQABAgMEBQYHCAkAAQI", state_hash: "BAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQ", receipts_hash: "BgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgY", proposer_public_key: "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc", signature: "CAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICA" }, transactions: [Transaction { from_address: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", to_address: "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE", value: 1000, tip: 1, gas_limit: 100000, gas_price: 100000, data: "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg", n_txs_on_chain_from_address: 0, hash: "BgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgY", signature: "CAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICA" }, Transaction { from_address: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", to_address: "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE", value: 1000, tip: 1, gas_limit: 100000, gas_price: 100000, data: "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg", n_txs_on_chain_from_address: 0, hash: "BgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgY", signature: "CAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICA" }], receipts: [Receipt { status_code: Success, gas_consumed: 100, return_value: [], events: [Event { topic: "\n\u{14}\u{1e}(2<", value: "\u{6}\u{2}\u{3}" }, Event { topic: "\n\u{14}\u{1e}(2<", value: "\u{6}\u{2}\u{3}" }] }, Receipt { status_code: Success, gas_consumed: 100, return_value: [], events: [Event { topic: "\n\u{14}\u{1e}(2<", value: "\u{6}\u{2}\u{3}" }, Event { topic: "\n\u{14}\u{1e}(2<", value: "\u{6}\u{2}\u{3}" }] }] }"#;

        assert_eq!(output, result);
        
        let transaction = protocol_types::Transaction {
            from_address : [0u8; 32],
            to_address : [1u8; 32],
            value : 1000,
            tip : 1,
            gas_limit : 100000,
            gas_price : 100000,
            data : vec![2u8; 100],
            n_txs_on_chain_from_address : 0,
            hash :[6u8; 32],
            signature : [8u8; 64]
        };
        let serialized_tx = protocol_types::Transaction::serialize(&transaction);
        let output = super::to_display_types(&serialized_tx, "tx");
        let result = r#"Transaction { from_address: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", to_address: "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE", value: 1000, tip: 1, gas_limit: 100000, gas_price: 100000, data: "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg", n_txs_on_chain_from_address: 0, hash: "BgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgY", signature: "CAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICA" }"#;

        assert_eq!(output, result);

        let receipt = protocol_types::Receipt {
            status_code: protocol_types::ReceiptStatusCode::Success,
            gas_consumed: 100 as u64,
            return_value: vec![],
            events: generate_events()
        };
        let serialized_recp = protocol_types::Receipt::serialize(&receipt);
        let output = super::to_display_types(&serialized_recp, "recp");
        let result = r#"Receipt { status_code: Success, gas_consumed: 100, return_value: [], events: [Event { topic: "\n\u{14}\u{1e}(2<", value: "\u{6}\u{2}\u{3}" }, Event { topic: "\n\u{14}\u{1e}(2<", value: "\u{6}\u{2}\u{3}" }] }"#;

        assert_eq!(output, result);


        let event = protocol_types::Event {
            // add data
            topic: vec![10,20,30,40,50,60],
            value: vec![6,2,3]
        };
        let serialized_evt = protocol_types::Event::serialize(&event);
        let output = super::to_display_types(&serialized_evt, "evt");
        let result = r#"Event { topic: "\n\u{14}\u{1e}(2<", value: "\u{6}\u{2}\u{3}" }"#;
        assert_eq!(output, result);


        let call = protocol_types::CallData {
            method_name: "call data".to_string(),
            arguments:   [1u8; 34].to_vec()            
        };
        let serialized_call = protocol_types::CallData::serialize(&call);
        let output = super::to_display_types(&serialized_call, "call");
        let result = r#"CallData { method_name: "call data", arguments: "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQ" }"#;
        assert_eq!(output, result);

        let merkle_proof = protocol_types::MerkleProof {
            // add data
            root_hash : [1u8; 32],
            total_leaves_count: 123,
            leaf_indices :vec![0,4,100],
            leaf_hashes : vec![[1u8; 32], [1u8; 32], [1u8; 32]],
            proof :[1u8; 128].to_vec()
        };
        let serialized_mrpf = protocol_types::MerkleProof::serialize(&merkle_proof);
        let output = super::to_display_types(&serialized_mrpf, "mprf");
        let result = r#"MerkleProof { root_hash: "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE", total_leaves_count: 123, leaf_indices: [0, 4, 100], leaf_hashes: ["AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE", "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE", "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE"], proof: [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1] }"#;
        assert_eq!(output, result);


        let state_proof = protocol_types::StateProofs {
            // add data
            root_hash : [1u8; 32],
            items : vec![
                ([1u8; 21].to_vec(), Some([1u8; 32].to_vec())), 
                ([1u8; 23].to_vec(), None), 
                ([1u8; 24].to_vec(), Some([1u8; 35].to_vec())), 
            ],
            proof : vec![[1u8; 56].to_vec(), [1u8; 57].to_vec(), [1u8; 58].to_vec()]
        };
        let serialized_sprfs = protocol_types::StateProofs::serialize(&state_proof);
        let output = super::to_display_types(&serialized_sprfs, "sprfs");
        let result = r#"StateProofs { root_hash: "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE", items: [([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], Some([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])), ([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], None), ([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], Some([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]))], proof: [[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]] }"#;
        assert_eq!(output, result);

    }

    fn generate_events() -> Vec<protocol_types::Event> {

        let mut ret = vec![];
        for _ in 0..2 {
            let event = protocol_types::Event {
                // add data
                topic: vec![10,20,30,40,50,60],
                value: vec![6,2,3]
            };
            ret.push(event);
        }
        ret
    }


    fn generate_receipts() -> Vec<protocol_types::Receipt> {
        let mut ret = vec![];
        for _ in 0..2 {
            let receipt = protocol_types::Receipt {
                status_code: protocol_types::ReceiptStatusCode::Success,
                gas_consumed: 100,
                return_value: vec![],
                events: generate_events()
            };
            ret.push(receipt);
        }
        ret
    }

    fn generate_transactions() -> Vec<protocol_types::Transaction> {
     
        let mut ret = vec![];
        for _ in 0..2 {
            let transaction = protocol_types::Transaction {
                from_address : [0u8; 32],
                to_address : [1u8; 32],
                value : 1000,
                tip : 1,
                gas_limit : 100000,
                gas_price : 100000,
                data : vec![2u8; 100],
                n_txs_on_chain_from_address : 0,
                hash :[6u8; 32],
                signature : [8u8; 64]
            };
            ret.push(transaction);
        }
        ret
    }
}