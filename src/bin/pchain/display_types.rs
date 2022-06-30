use protocol_types::transaction::{Receipt};

// Displayed Types module translate original raw data types: Transaction/ Block to human readble form.
// For example, data which are originally in bytes will be displayed in base64 encoded string.

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
            from_address: base64::encode(transaction.from_address),
            to_address: base64::encode(transaction.to_address),
            value: transaction.value,
            tip: transaction.tip,
            gas_limit: transaction.gas_limit,
            gas_price: transaction.gas_price,
            data: base64::encode(transaction.data),
            n_txs_on_chain_from_address: transaction.n_txs_on_chain_from_address,
            hash: base64::encode(transaction.hash),
            signature: base64::encode(transaction.signature),
        }
    }
}

#[derive(Debug)]
pub struct BlockHeader {
    pub blockchain_id: u64,
    pub block_version_number: u64,
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
            timestamp: blockheader.timestamp,
            prev_block_hash: base64::encode(blockheader.prev_block_hash),
            this_block_hash: base64::encode(blockheader.this_block_hash),
            txs_hash: base64::encode(blockheader.txs_hash),
            state_hash: base64::encode(blockheader.state_hash),
            receipts_hash: base64::encode(blockheader.receipts_hash),
            proposer_public_key: base64::encode(blockheader.proposer_public_key),
            signature: base64::encode(blockheader.signature),
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
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub events: Vec<Event>,
    pub receipts: Vec<Receipt>,
}

impl From<protocol_types::block::Block> for Block {
    fn from(block: protocol_types::block::Block) -> Block {
        let txs_beautified: Vec<Transaction> = block.transactions.into_iter().map(
            |protocol_type_transaction|
            From::<protocol_types::transaction::Transaction>::from(protocol_type_transaction)
        ).collect();
        let events_beautified: Vec<Event> = block.receipts.iter().flat_map(|receipt|{
            let events = receipt.events.to_vec();
            let return_events :Vec<Event> = events.into_iter().map(|event|{
                From::<protocol_types::transaction::Event>::from(event)
            }).collect();
            return_events
        }).collect();
        Block {
            header: From::<protocol_types::block::BlockHeader>::from(block.header),
            transactions: txs_beautified,
            events: events_beautified,
            receipts: block.receipts,

        }
    }
}

#[derive(Debug)]
pub struct Blocks {
    pub blocks: Vec<Block>,
}

impl From<protocol_types::block::Blocks> for Blocks {
    fn from(blocks_reply: protocol_types::block::Blocks) -> Blocks {
        let blocks_beautified: Vec<Block> = blocks_reply.blocks.into_iter().map(
            |protocol_type_block|
            From::<protocol_types::block::Block>::from(protocol_type_block)
        ).collect();
        Blocks {
            blocks: blocks_beautified,
        }
    }
}