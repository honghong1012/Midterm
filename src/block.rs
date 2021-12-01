use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256, Hashable};
use crate::blockchain::Blockchain;
use crate::crypto::merkle::*;
use crate::transaction::*;
use ring::digest::*;
use rand::Rng;
use std::time::{ SystemTime, UNIX_EPOCH };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: Header,
    pub data: Vec<SignedTansaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub parent: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: u128,
    pub merkle_root: H256,
}

impl Hashable for Transaction{
    fn hash(&self) -> H256 {
        let transaction_bytes = bincode::serialize(self).unwrap();
        let result = ring::digest::digest(&ring::digest::SHA256, &transaction_bytes);
        let transaction_hash = result.into();
        return transaction_hash;
    }
}

impl Hashable for SignedTansaction{
    fn hash(&self) -> H256 {
        let transaction_bytes = bincode::serialize(self).unwrap();
        let result = ring::digest::digest(&ring::digest::SHA256, &transaction_bytes);
        let transaction_hash = result.into();
        return transaction_hash;
    }
}

impl Hashable for Header {
    fn hash(&self) -> H256 {
        let header_bytes = bincode::serialize(self).unwrap();
        let result = ring::digest::digest(&ring::digest::SHA256, &header_bytes);
        let header_hash = result.into();
        return header_hash;
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        let block_hash = self.header.hash();
        return block_hash;
    }
}

pub fn now() -> u128{
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
    ;
    duration.as_secs() as u128 * 1000 + duration.subsec_millis() as u128
}

impl Block{
    pub fn new (header:Header, data:Vec<SignedTansaction>) -> Self{
        Block{
            header,
            data,
        }
    }
}

impl Header{
    pub fn new (parent: H256, nonce: u32, difficulty: H256, timestamp: u128, merkle_root: H256,) -> Self{
        Header{
            parent,
            nonce,
            difficulty,
            timestamp,
            merkle_root,
        }
    }
}

#[cfg(any(test, test_utilities))]
pub mod test {
    use super::*;
    use crate::crypto::hash::H256;

    pub fn generate_random_block(parent: &H256) -> Block {
        // generate the random transactions
        let transaction: Vec<Transaction> = vec![Default::default(), Default::default()];
        // use merkle tree to caculate the root of the transactions
        let merkle_tree = MerkleTree::new(&transaction); 
        let root = merkle_tree.root();
        let merkle_root = root;
        // create random nonce
        let nonce = rand::thread_rng().gen_range(1,5000);
        // timestamp
        let timestamp = now();
        // clone parent hash value
        let parent = parent.clone();
        // temporarily use the type H256 as difficulty
        let difficulty = parent.clone();
        // generate new header
        let new_header = Header::new(parent, nonce, difficulty, timestamp, merkle_root);
        let header = new_header;
        let data = transaction;
        // generate new block
        let new_block = Block::new(header, data);
        return new_block;
    }
}
