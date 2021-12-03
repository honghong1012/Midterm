use super::message::Message;
use super::peer;
use crate::network::server::Handle as ServerHandle;
use crate::crypto::hash:: Hashable;
use crossbeam::channel;
use log::{debug, warn};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::blockchain::*;
use crate::block::*;
use crate::transaction::*;
use std::thread;
use log::info;

#[derive(Clone)]
pub struct Context {
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
}

pub fn new(
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<Mempool>>,
) -> Context {
    Context {
        msg_chan: msg_src,
        num_worker,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
        mempool: Arc::clone(mempool)
    }
}

impl Context {
    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        // create hashmap to store the parent-losting block
        let mut buffer_hash = HashMap::new();
        let mut buffer_block = HashMap::new();
        let mut delay_list = Vec::new();
        loop {
            let msg = self.msg_chan.recv().unwrap();
            let (msg, peer) = msg;
            let mut blc = self.blockchain.lock().unwrap();
            let mut mp = self.mempool.lock().unwrap();
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    // if receive the Ping message
                    // print the Ping message
                    // return the Pong message to peer
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }

                Message::Pong(nonce) => {
                    // print the Pong message
                    debug!("Pong: {}", nonce);
        
                }

                Message::NewBLockHashes(newblockhashes) => {
                    // if get newblockhashes message and the hashes not in the blockchain
                    // return getblocks message
                    let mut lost_block = Vec::new();
                    info!("get blockhashes!(w)");
                    for hash in &newblockhashes{
                        if !blc.blocks.contains_key(&hash){
                            lost_block.push(hash.clone()); 
                        }
                    }
                    peer.write(Message::GetBlocks(lost_block));
                }

                Message::GetBlocks(blockhashes) => {
                    // if get getblocks message and the hashes in ur blockchain
                    // return blocks message
                    let mut exisited_hashes = Vec::new();
                    info!("get getblocks mess!");
                    for hash in &blockhashes{
                        if blc.blocks.contains_key(&hash){
                            let block_info = blc.blocks.get(&hash).expect("failed");
                            exisited_hashes.push(block_info.clone());
                        }
                    }
                    peer.write(Message::Blocks(exisited_hashes));
                }

                Message::Blocks(blocks) => {
                    // if get blocks message
                    // insert the blocks into blockchain
                    let mut new_blocks = Vec::new();
                    let mut lost_block = Vec::new();
                    for block in &blocks{
                        let hash = &block.hash();
                        if !blc.blocks.contains_key(hash){// if the block doesn't exisit in the blockchain
                            // Checks
                            // 1. pow check 2. parent check 3. orphan block handler
                            // PoW check
                            if !buffer_block.contains_key(hash){
                                // first time receive this block,calculate the delay
                                let recevie_time = now();
                                let b_tsp = block.header.timestamp.clone();
                                let block_delay = recevie_time - b_tsp;
                                info!("block delay: {} (w)", &block_delay);
                                // compute the average delay
                                delay_list.push(block_delay);
                                let average_delay : u128 = delay_list.iter().sum();
                                let average_delay = average_delay / (delay_list.len() as u128);
                                info!("Average block delay:{}(w)", &average_delay);

                            }
                            let now_block = block.clone();
                            let difficulty = block.header.difficulty.clone();
                            if hash > &difficulty{
                                // if pow doesn't match, just ignore this new block
                                continue;
                            }
                            // use the parent's hash to find parent's height
                            let parent_hash = block.header.parent.clone();
                            // if the parent has not inserted, we cannot find the height in the hash map
                            // need to discord the panic situation
                            // Parent check
                            let parent_result = blc.heights.get(&parent_hash);
                            let mut now_height = 0;
                            let mut flag = 0;
                            match parent_result{
                                None => flag = 1,
                                Some(v) => now_height = v + 1,
                            }
                            if flag == 1{
                                // key:parent_hash value:child_hash
                                buffer_hash.insert(parent_hash.clone(), hash.clone());
                                // key:child_hash value:child_block
                                buffer_block.insert(hash.clone(), now_block.clone());
                                // lost_block store what we have hash but no blocks
                                lost_block.push(parent_hash.clone());
                                // send a get block message
                                peer.write(Message::GetBlocks(lost_block.clone()));
                                continue;
                            }
                            let parent_block = blc.blocks.get(&parent_hash).expect("failed");
                            let parent_difficulty = parent_block.header.difficulty.clone();
                            if difficulty != parent_difficulty{
                                // if difficulty doesn't match, just ignore this new block
                                continue;
                            }
                            // if blocks have parents then insert block and height
                            blc.heights.insert(hash.clone(), now_height);
                            blc.blocks.insert(hash.clone(), now_block);
                            new_blocks.push(hash.clone());
                            // after inserting a new block, we need to look through
                            // if the new block is some orphan's parents
                            // turn into iteratively
                            let mut p_hash = hash.clone();
                            while buffer_hash.contains_key(&p_hash){
                                let child_hash = buffer_hash.get(&p_hash).expect("failed").clone();
                                let child_block = buffer_block.get(&child_hash).expect("failed");
                                let new_p_height = blc.heights.get(&p_hash).expect("failed");
                                let child_height = new_p_height + 1;
                                blc.blocks.insert(child_hash.clone(), child_block.clone());
                                blc.heights.insert(child_hash.clone(), child_height);
                                // after inserting the lost hash in buffer
                                // need to broadcast it and delete it from the buffer
                                new_blocks.push(child_hash.clone());
                                buffer_block.remove(&child_hash);
                                buffer_hash.remove(&p_hash);
                                p_hash = child_hash.clone()
                            }
                            // need to broadcast after inserted a new block
                            self.server.broadcast(Message::NewBLockHashes(new_blocks.clone()));
                            peer.write(Message::NewBLockHashes(new_blocks.clone()));
                            // count the numbers of block in the blockchain
                            let tip = blc.tip();
                            let num_in_blc = blc.heights.get(&tip).expect("failed");
                            info!("We have {} blocks in our blockchain(w)", &num_in_blc);
                        }
                    }
                    
                }

                Message::NewTransactionHashes(newtxhashes) => {
                    let mut lost_tx = Vec::new();
                    for hash in &newtxhashes{
                        if !mp.valid_tx.contains_key(&hash){
                            lost_tx.push(hash.clone());
                        }
                    }
                    peer.write(Message::GetTransactions(lost_tx));
                    info!("new transaction hashes received(w)");//test
                }

                Message::GetTransactions(txhashes) => {
                    let mut exisited_hashes = Vec::new();
                    for hash in &txhashes{
                        if mp.valid_tx.contains_key(&hash){
                            let tx_info = mp.valid_tx.get(&hash).expect("failed");
                            exisited_hashes.push(tx_info.clone());
                        }
                    }
                    peer.write(Message::Transactions(exisited_hashes));
                    info!("To get tx(w)");
                }

                Message::Transactions(signedtransactions) => {
                    // if get transactions, do checks
                    // 1.check if signature is signed correctly by the public key
                    // 2.double spending check
                    // 3.when get blocks, check transactions again(send message to get transactions info)
                    // mempool operations!
                    info!("got new tx!(w)");
                    for signedtx in &signedtransactions{
                        let transaction = &signedtx.tx;
                        let sig = &signedtx.signature;
                        let public_key = &signedtx.public_key;
                        // if the transaciton valid and not in mempool(not included in block)
                        if verify(transaction, public_key.clone(), sig.clone()){
                            if !mp.valid_tx.contains_key(&transaction.hash()){
                                mp.valid_tx.insert(transaction.hash().clone(),signedtx.clone());
                                info!("new tx in pool!");
                            }
                        }
                    }
                }
                
            }
            drop(blc);
            drop(mp);
        }
    }
}
