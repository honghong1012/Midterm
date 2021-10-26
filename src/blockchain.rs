use crate::block::{Block, Header};
use crate::crypto::hash::{H256, Hashable};
use std::collections::HashMap;
use super::*;

pub struct Blockchain{
    // use hashmap to save the blocks and heights
    pub blocks: HashMap<H256, Block>,
    pub heights: HashMap<H256, u32>,
    // pub tip: H256,
}


impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        // initiate the header and data for the genesis block
        let data = vec![];
        let parent = [0;32].into();
        let nonce = 1;
        let difficulty = [0;32].into();
        let timestamp = 2;
        let merkle_root = [0;32].into();
        let genesis_header = Header{parent, nonce, difficulty, timestamp, merkle_root};
        let header = genesis_header;
        let genesis = Block{header, data};
        let genesis_hash = genesis.hash();
        let mut blocks = HashMap::new();
        let mut heights = HashMap::new();
        // for the genesis block, the height must be 0
        blocks.insert(genesis_hash, genesis);
        heights.insert(genesis_hash, 0);
        Blockchain{
            blocks,
            heights,
        }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        // insert the new block into blockchain
        let now_hash = block.hash();
        // use the parent's hash to find parent's height
        let parent_hash = block.header.parent.clone();
        let parent_height = self.heights.get(&parent_hash).expect("failed");
        let now_height = parent_height + 1;
        // deep copy the block
        let now_block = block.clone();
        self.heights.insert(now_hash.clone(), now_height);
        self.blocks.insert(now_hash, now_block);
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        let mut tip = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0].into();
        let mut last = 0;
        for(hash, height) in &self.heights{
            if height >= &last{
                last = height.clone();
                tip = *hash;
            }
        }
        return tip;
    }

    /// Get all blocks in longest chain
    #[cfg(any(test, test_utilities))]
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        // find the tip, and do reverse travel
        let mut tip = self.tip().clone();
        let mut longest_chain_reverse = Vec::new();
        let mut longest_chain = Vec::new();
        if self.heights.get(&tip).expect("failed") > &0 {
            while self.heights.get(&tip).expect("failed") > &0{
                longest_chain_reverse.push(tip);
                tip = self.blocks.get(&tip).expect("failed").header.parent;
            }
            longest_chain_reverse.push(tip);
            for i in (0..longest_chain_reverse.len()).rev(){
                longest_chain.push(longest_chain_reverse[i]);
            }
        }    
        else{
            longest_chain.push(tip);
        }
        return longest_chain;
    }
}

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::block::test::generate_random_block;
    use crate::crypto::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());
    }

    #[test]
    fn insert_several() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        let block2 = generate_random_block(&genesis_hash);
        let block3 = generate_random_block(&block2.hash());
        let block4 = generate_random_block(&block.hash());
        let block5 = generate_random_block(&block3.hash());
        blockchain.insert(&block);
        blockchain.insert(&block2);
        blockchain.insert(&block3);
        blockchain.insert(&block4);
        blockchain.insert(&block5);
        assert_eq!(blockchain.tip(), block5.hash());
    }

    #[test]
    fn verify_several() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        let block2 = generate_random_block(&genesis_hash);
        let block3 = generate_random_block(&block2.hash());
        let block4 = generate_random_block(&block.hash());
        let block5 = generate_random_block(&block3.hash());
        blockchain.insert(&block);
        blockchain.insert(&block2);
        blockchain.insert(&block3);
        blockchain.insert(&block4);
        blockchain.insert(&block5);
        let result = blockchain.all_blocks_in_longest_chain();
        for i in 0..result.len() {
            println!("{}", result[i]);
        }
        assert_eq!(result, vec![ genesis_hash, block2.hash(), block3.hash(),  block5.hash()]);
    }

    #[test]
    fn verify_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);

        blockchain.insert(&block);

        let result = blockchain.all_blocks_in_longest_chain();
        for i in 0..result.len() {
            println!("{}", result[i]);
        }
        assert_eq!(result, vec![ genesis_hash, block.hash()]);
    }

    #[test]
    fn verify_zero() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        //let block = generate_random_block(&genesis_hash);

        //blockchain.insert(&block);

        let result = blockchain.all_blocks_in_longest_chain();
        for i in 0..result.len() {
            println!("{}", result[i]);
        }
        assert_eq!(result, vec![genesis_hash]);
    }

}
