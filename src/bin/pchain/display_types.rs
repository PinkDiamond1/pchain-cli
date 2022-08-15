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
// Displayed Types module translate original raw data types: Transaction/ Block to human readble form.
// For example, data which are originally in bytes will be displayed in base64 encoded string.

use serde::{Deserialize, Serialize};

use crate::{Base64String};

#[derive(Debug)]
pub struct Transaction {
    pub from_address: String,
    pub to_address: String,
    pub value: u64,
    pub tip: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub data: String,
    pub n_txs_on_chain_from_address: u64,
    pub hash: String,
    pub signature: String,
}

impl From<protocol_types::transaction::Transaction> for Transaction {
    fn from(transaction: protocol_types::transaction::Transaction) -> Transaction {
        Transaction {
            from_address: protocol_types::Base64URL::encode(transaction.from_address).to_string(),
            to_address: protocol_types::Base64URL::encode(transaction.to_address).to_string(),
            value: transaction.value,
            tip: transaction.tip,
            gas_limit: transaction.gas_limit,
            gas_price: transaction.gas_price,
            data: protocol_types::Base64URL::encode(transaction.data).to_string(),
            n_txs_on_chain_from_address: transaction.n_txs_on_chain_from_address,
            hash: protocol_types::Base64URL::encode(transaction.hash).to_string(),
            signature: protocol_types::Base64URL::encode(transaction.signature).to_string(),
        }
    }
}

#[derive(Debug)]
pub struct BlockHeader {
    pub blockchain_id: u64,
    pub block_version_number: u64,
    pub block_number: u64,
    pub timestamp: u32,
    pub prev_block_hash: String,
    pub this_block_hash: String,
    pub txs_hash: String,
    pub state_hash: String,
    pub receipts_hash: String,
    pub proposer_public_key: String,
    pub signature: String,
}

impl From<protocol_types::block::BlockHeader> for BlockHeader {
    fn from(blockheader: protocol_types::block::BlockHeader) -> BlockHeader {
        BlockHeader {
            blockchain_id: blockheader.blockchain_id,
            block_version_number: blockheader.block_version_number,
            block_number: blockheader.block_number,
            timestamp: blockheader.timestamp,
            prev_block_hash: protocol_types::Base64URL::encode(blockheader.prev_block_hash).to_string(),
            this_block_hash: protocol_types::Base64URL::encode(blockheader.this_block_hash).to_string(),
            txs_hash: protocol_types::Base64URL::encode(blockheader.txs_hash).to_string(),
            state_hash: protocol_types::Base64URL::encode(blockheader.state_hash).to_string(),
            receipts_hash: protocol_types::Base64URL::encode(blockheader.receipts_hash).to_string(),
            proposer_public_key: protocol_types::Base64URL::encode(blockheader.proposer_public_key).to_string(),
            signature: protocol_types::Base64URL::encode(blockheader.signature).to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Event {
    pub topic: String,
    pub value: String
}
impl From<protocol_types::transaction::Event> for Event {
    fn from(event: protocol_types::transaction::Event) -> Event {
        Event {
            topic: String::from_utf8(event.topic).unwrap(),
            value: String::from_utf8(event.value).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct Receipt {
    pub status_code: protocol_types::receipt_status_codes::ReceiptStatusCode,
    pub gas_consumed: u64,
    pub return_value: Vec<u8>,
    pub events: Vec<Event>,
}

impl From<protocol_types::Receipt> for Receipt {
    fn from(receipt: protocol_types::transaction::Receipt) -> Receipt {

        let events_beautified: Vec<Event> = receipt.events.into_iter().map(
            |protocol_types_event|{
                From::<protocol_types::transaction::Event>::from(protocol_types_event)
        }).collect();
        Receipt {
            status_code: receipt.status_code,
            gas_consumed: receipt.gas_consumed,
            return_value: receipt.return_value,
            events: events_beautified,
        }
    }
}

#[derive(Debug)]
pub struct TransactionWithReceipt {
    pub tx_num: u64,
    pub transaction: Transaction,
    pub receipt: Receipt
}

#[derive(Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub receipts: Vec<Receipt>,
}

impl From<protocol_types::block::Block> for Block {
    fn from(block: protocol_types::block::Block) -> Block {
        let txs_beautified: Vec<Transaction> = block.transactions.into_iter().map(
            |protocol_type_transaction|
            From::<protocol_types::transaction::Transaction>::from(protocol_type_transaction)
        ).collect();
        let receipt_beautified: Vec<Receipt> = block.receipts.into_iter().map(
            |protocol_type_receipt|
            From::<protocol_types::transaction::Receipt>::from(protocol_type_receipt)
        ).collect();
        
        Block {
            header: From::<protocol_types::block::BlockHeader>::from(block.header),
            transactions: txs_beautified,
            receipts: receipt_beautified,
        }
    }
}

#[derive(Debug)]
pub struct CallData {
    pub method_name: String,
    pub arguments: Base64String
}

impl From<protocol_types::sc_params::CallData> for CallData {
    fn from(call_data: protocol_types::sc_params::CallData) -> CallData {
        CallData { 
            method_name: call_data.method_name ,
            arguments: protocol_types::Base64URL::encode(call_data.arguments).to_string(),
        }
    }
}

#[derive(Debug)]
pub struct MerkleProof {
    pub root_hash : Base64String,
    pub total_leaves_count: usize,
    pub leaf_indices :Vec<usize>,
    pub leaf_hashes :Vec<Base64String>,
    pub proof :Vec<u8>,
}

impl From<protocol_types::MerkleProof> for MerkleProof {
    fn from(merkle_proof: protocol_types::MerkleProof) -> MerkleProof {
        let leaf_hashes: Vec<Base64String> = merkle_proof.leaf_hashes.iter().map(
            |h|
            protocol_types::Base64URL::encode(h).to_string()
        ).collect();
        MerkleProof {
            root_hash: protocol_types::Base64URL::encode(merkle_proof.root_hash).to_string(),
            total_leaves_count: merkle_proof.total_leaves_count,
            leaf_indices: merkle_proof.leaf_indices,
            leaf_hashes,
            proof: merkle_proof.proof,
        }
    }
}

#[derive(Debug)]
pub struct StateProofs {
    pub root_hash: Base64String,
    pub items: Vec<protocol_types::StateProofItem>,
    pub proof: protocol_types::StateProof
}

impl From<protocol_types::StateProofs> for StateProofs {
    fn from(state_proofs: protocol_types::StateProofs) -> StateProofs {

        StateProofs {
            root_hash: protocol_types::Base64URL::encode(state_proofs.root_hash).to_string(),
            items: state_proofs.items,
            proof: state_proofs.proof,
        }
    }
}

#[derive(Debug)]
pub struct Blocks {
    pub blocks: Vec<Block>,
}

impl From< Vec::<protocol_types::Block> > for Blocks {
    fn from(block_vec: Vec::<protocol_types::Block>) -> Self {
        let blocks_beautified: Vec<Block> = block_vec.into_iter().map(
            |protocol_type_block|{
            let block_without_blocknum:Block = From::<protocol_types::Block>::from(protocol_type_block);
            block_without_blocknum
        }).collect();
        Blocks {
            blocks: blocks_beautified,
        }
    }
}

#[derive(Debug)]
pub struct BlockHeaders {
    pub block_headers: Vec<BlockHeader>,
}

impl From< Vec::<protocol_types::BlockHeader> > for BlockHeaders {
    fn from(block_vec: Vec::<protocol_types::BlockHeader>) -> Self {
        let blockheaders_beautified: Vec<BlockHeader> = block_vec.into_iter().map(
            |protocol_type_blockheader|{
            let block_header:BlockHeader = From::<protocol_types::BlockHeader>::from(protocol_type_blockheader);
            block_header
        }).collect();
        BlockHeaders {
            block_headers: blockheaders_beautified,
        }
    }
}

#[derive(Debug)]
pub struct TransactionNum {
    pub umber: u64
}

#[derive(Debug)]
pub struct TransactionsWithReceipt {
    pub transactions: Vec<TransactionWithReceipt>,
}

impl From< Vec::<(u64, protocol_types::Transaction, protocol_types::Receipt)> > for TransactionsWithReceipt {
    
    fn from(tx_vector: Vec::<(u64, protocol_types::Transaction, protocol_types::Receipt)>) -> TransactionsWithReceipt {
        let mut transactions = vec![];
        
        tx_vector.iter().for_each(|(tx_num, ptype_tx, ptype_recp)|{
            let tx: Transaction = From::<protocol_types::Transaction>::from(ptype_tx.clone());
            let recp: Receipt = From::<protocol_types::Receipt>::from(ptype_recp.clone());
            transactions.push(TransactionWithReceipt{
                tx_num: tx_num.clone(),
                transaction: tx,
                receipt: recp
            });

        });
        TransactionsWithReceipt { transactions }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockSummary {
    pub height: u64,
    pub block_hash: String,
    pub state_hash: String,
    pub receipts_hash: String,
    pub time: u32,
    pub tx_count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TxnSummary {
    pub number: u64,
    pub hash: String,
    pub from_address: String,
    pub to_address: String,
    pub value: u64,
    pub status_code: u8,
}