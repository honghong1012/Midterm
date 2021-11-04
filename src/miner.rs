use crate::network::server::Handle as ServerHandle;
use crate::network::message::Message;
use log::info;
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::blockchain::*;
use crate::block::*;
use crate::transaction::*;
use crate::crypto::merkle::*;
use crate::crypto::hash::Hashable;

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    
    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Miner shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i) => {
                info!("Miner starting in continuous mode with lambda {}", i);
                self.operating_state = OperatingState::Run(i);
            }
        }
    }

    fn miner_loop(&mut self) {
        // main mining loop
        let mut block_num = 0;
        // connect server to broadcast
        let server = self.server.clone();
        let mut newblockhashes = Vec::new();
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    self.handle_control_signal(signal);
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        self.handle_control_signal(signal);
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // TODO: actual mining
            // let mut blc = self.blockchain.lock().unwrap();
            // // return the final block hash in the longest chain 
            // let parent = blc.tip();
            // // initial the block
            // let difficulty = blc.blocks.get(&parent).expect("failed").header.difficulty;
            // let timestamp = now();
            // // random content
            // let transaction = vec![
            //     Transaction{
            //         x: 1,
            //         y: 1,
            //     }
            // ];
            // let nonce = 0;
            // let merkle_tree = MerkleTree::new(&transaction); 
            // let merkle_root = merkle_tree.root();
            // let data = transaction.clone();
            // let header = Header::new(parent, nonce, difficulty, timestamp, merkle_root);
            // let mut block = Block::new(header, data);

            // increment nounce
            for nonce_attempt in 0..(u32::max_value()){
                // everytime to calculate the nounce we need to access the lock(in 'for' loop or out of 'for' loop?)
                let mut blc = self.blockchain.lock().unwrap();
                // return the final block hash in the longest chain 
                let parent = blc.tip();
                // initial the block
                let difficulty = blc.blocks.get(&parent).expect("failed").header.difficulty;
                // random content
                let transaction = vec![
                    Transaction{
                      x: 1,
                      y: 1,
                     }
                ];
                let nonce = 0;
                let merkle_tree = MerkleTree::new(&transaction); 
                let merkle_root = merkle_tree.root();
                let data = transaction.clone();
                // we cannot change the timestamp after the hash has been caculated
                let timestamp = now();
                let header = Header::new(parent, nonce, difficulty, timestamp, merkle_root);
                let mut block = Block::new(header, data);
                block.header.nonce = nonce_attempt;
                // calculate the hash and compare the difficulty
                let hash = block.hash();
                // if match, the block is mined successfully
                if hash <= difficulty{
                    // insert the block into blockchain
                    blc.insert(&block);
                    // change the blocknum
                    block_num = block_num + 1;
                    // broadcast the new block hashes to peer
                    newblockhashes.push(hash);
                    server.broadcast(Message::NewBLockHashes(newblockhashes.clone()));
                    // print the timestamp and number of blocks mined
                    info!("Successfully mine {} block(s)", &block_num);
                    info!("Timestamp:{}", &block.header.timestamp);
                    let tip = blc.tip();
                    let num_in_blc = blc.heights.get(&tip).expect("failed");
                    info!("We have {} blocks in our blockchain(m)", &num_in_blc);
                }
            }

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }
}
