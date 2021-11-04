use super::message::Message;
use super::peer;
use crate::network::server::Handle as ServerHandle;
use crate::crypto::hash:: Hashable;
use crossbeam::channel;
use log::{debug, warn};
use std::sync::{Arc, Mutex};
use crate::blockchain::*;
use std::thread;
use log::info;

#[derive(Clone)]
pub struct Context {
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
}

pub fn new(
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>,
) -> Context {
    Context {
        msg_chan: msg_src,
        num_worker,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
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
        loop {
            let msg = self.msg_chan.recv().unwrap();
            let (msg, peer) = msg;
            let mut blc = self.blockchain.lock().unwrap();
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
                    for block in &blocks{
                        let hash = &block.hash();
                        if !blc.blocks.contains_key(hash){// if the block doesn't exisit in the blockchain
                            let now_block = block.clone();
                            // use the parent's hash to find parent's height
                            let parent_hash = block.header.parent.clone();
                            // if the parent has not inserted, we cannot find the height in the hash map
                            // need to discord the panic situation
                            // let parent_height = blc.heights.get(&parent_hash).expect("failed");
                            let parent_result = blc.heights.get(&parent_hash);
                            let now_height;
                            match parent_result{
                                None => continue,
                                Some(v) => now_height = v + 1,
                            }
                            // insert height and block
                            blc.heights.insert(hash.clone(), now_height);
                            blc.blocks.insert(hash.clone(), now_block);
                            new_blocks.push(hash.clone());
                            
                            // need to broadcast after inserted a new block
                            peer.write(Message::NewBLockHashes(new_blocks.clone()));
                            // count the numbers of block in the blockchain
                            let tip = blc.tip();
                            let num_in_blc = blc.heights.get(&tip).expect("failed");
                            info!("We have {} blocks in our blockchain(w)", &num_in_blc);
                        }
                    }
                    
                }
            }
        }
    }
}
