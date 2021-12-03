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
    mempool: Arc<Mutex<Mempool>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<Mempool>>,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    
    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
        mempool: Arc::clone(mempool)
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
            // // everytime to calculate the nounce we need to access the lock
            // let mut blc = self.blockchain.lock().unwrap();
            // let mut mp = self.mempool.lock().unwrap();
            // let mut transaction = Vec::new();
            // let mut existed_hashes = Vec::new();
            // let mut siggg = Vec::new();
            // let mut pubbb = Vec::new();
            // siggg.push(0);
            // pubbb.push(0);
            // // return the final block hash in the longest chain 
            // let parent = blc.tip();
            // let mut block_size = 0;
            // // initial the block
            // let difficulty = blc.blocks.get(&parent).expect("failed").header.difficulty;
            // let transaction = 
            //         Transaction{
            //            recipient_address:[0;20].into(),
            //            value:1, 
            //            account_nonce:1,
            //          };
            // let signedtx = vec![
            //         SignedTransaction{
            //             tx:transaction,
            //             signature:siggg,
            //             public_key:pubbb,
            //         }
            // ];
            
            // mempool empty
            // while mp.valid_tx.is_empty(){
            //     let l = 1;
            // }
            // // mempool not empty
            // for (txhashes, tx) in &mp.valid_tx{
            //     transaction.push(tx.clone());
            //     block_size += 1;
            //     existed_hashes.push(txhashes.clone());
            //     info!("catch one tx!(m)");
            //     // limit block size
            //     if block_size == 1{
            //         break;
            //     }
            // }
            // increment nounce
            for nonce_attempt in 0..(u32::max_value()){
                // everytime to calculate the nounce we need to access the lock(in 'for' loop or out of 'for' loop?)
                let mut transaction = Vec::new();
                let mut existed_hashes = Vec::new();
                // return the final block hash in the longest chain 
                let mut blc = self.blockchain.lock().unwrap();
                let parent = blc.tip();
                let mut block_size = 0;
                // initial the block
                let difficulty = blc.blocks.get(&parent).expect("failed").header.difficulty;
                drop(blc);
                // test part
                // let mut siggg = Vec::new();
                // let mut pubbb = Vec::new();
                // siggg.push(0);
                // pubbb.push(0);
                // let transaction = 
                //     Transaction{
                //        recipient_address:[0;20].into(),
                //        value:1, 
                //        account_nonce:1,
                //      };
                // let signedtx = vec![
                //     SignedTransaction{
                //         tx:transaction,
                //         signature:siggg,
                //         public_key:pubbb,
                //     }
                //  ];
                // random content
                // let transaction = vec![
                //     Transaction{
                //        recipient_address:[0;20].into(),
                //        value:1, 
                //        account_nonce:1,
                //      }
                // ];
                let mut mp = self.mempool.lock().unwrap();
                while mp.valid_tx.is_empty(){
                    let l = 1;
                }
                for (txhashes, tx) in &mp.valid_tx{
                    transaction.push(tx.clone());
                    block_size += 1;
                    existed_hashes.push(txhashes.clone());
                    // limit block size
                    if block_size == 2{
                        break;
                    }
                }
                drop(mp);
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
                    // delete the tx in mempool
                    // update the state
                    let mut mp = self.mempool.lock().unwrap();
                    let mut blc = self.blockchain.lock().unwrap();
                    for txhashes in &existed_hashes{
                        let signedtx = mp.valid_tx.get(txhashes).expect("failed");
                        let tx = &signedtx.tx;
                        let account = &signedtx.public_key;
                        let spend_value = tx.value;
                        let an_new = tx.account_nonce;

                        let state = &blc.state;
                        let mut public_bytes = [0; 20];
                        public_bytes.copy_from_slice(&account);
                        let (an_state,b) = state.get(&public_bytes.into()).expect("failed").clone();
                        blc.state.remove(&public_bytes.into());
                        blc.state.insert(public_bytes.into(),(an_new, b-spend_value));
                        mp.valid_tx.remove(&txhashes);
                    }
                    drop(mp);
                    drop(blc);

                    // insert the block into blockchain
                    let mut blc = self.blockchain.lock().unwrap();
                    blc.insert(&block);
                    // change the blocknum
                    block_num = block_num + 1;
                    // broadcast the new block hashes to peer
                    newblockhashes.push(hash);
                    self.server.broadcast(Message::NewBLockHashes(newblockhashes.clone()));
                    // print the length of the block
                    let block_size: Vec<u8> = bincode::serialize(&block).unwrap();
                    info!("Current block size is {}", &block_size.len());
                    // print the timestamp and number of blocks mined
                    info!("Successfully mine {} block(s)", &block_num);
                    info!("Timestamp:{}", &block.header.timestamp);
                    let tip = blc.tip();
                    let num_in_blc = blc.heights.get(&tip).expect("failed");
                    for tx in transaction{
                        let trans = tx.tx;
                        info!("receiver:{},value:{},account_nonce:{}",trans.recipient_address,trans.value,trans.account_nonce);
                    }
                    info!("We have {} blocks in our blockchain(m)", &num_in_blc);
                    drop(blc);
                    break;
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
