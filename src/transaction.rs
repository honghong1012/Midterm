use serde::{Serialize,Deserialize};
use ring::signature::{self,Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use rand::{Rng};
use crate::crypto::hash::{H160,H256};
use std::collections::HashMap;
use crate::network::server::Handle as ServerHandle;
use crate::network::message::Message;
use log::info;
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::blockchain::*;
use crate::block::*;
use crate::crypto::merkle::*;
use crate::crypto::hash::Hashable;
use crate::crypto::key_pair;


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    // put anyting in the transaction
    // pub x: i32,
    // pub y: i32,
    pub recipient_address:H160,
    pub value:u32, 
    pub account_nonce:u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignedTransaction {
    // put the transaction and signature together
    pub tx:Transaction,
    pub signature:Vec<u8>,//?
    pub public_key:Vec<u8>,//?
}


impl Transaction{
    pub fn new (recipient_address:H160, value:u32,account_nonce:u8) -> Self{
        Transaction{
            recipient_address,
            value,
            account_nonce,
        }
    }
}

impl SignedTransaction{
    pub fn new (tx:Transaction,signature:Vec<u8>,public_key:Vec<u8>) -> Self{
        SignedTransaction{
            tx,
            signature,
            public_key,
        }
    }
}

impl Hashable for Transaction{
    fn hash(&self) -> H256 {
        let transaction_bytes = bincode::serialize(self).unwrap();
        let result = ring::digest::digest(&ring::digest::SHA256, &transaction_bytes);
        let transaction_hash = result.into();
        return transaction_hash;
    }
}

impl Hashable for SignedTransaction{
    fn hash(&self) -> H256 {
        let transaction_bytes = bincode::serialize(self).unwrap();
        let result = ring::digest::digest(&ring::digest::SHA256, &transaction_bytes);
        let transaction_hash = result.into();
        return transaction_hash;
    }
}

pub struct Mempool {
    pub valid_tx: HashMap<H256,SignedTransaction>,
}

impl Mempool{
    pub fn new() -> Self{
        let newhashmap = HashMap::new();
        Mempool{
            valid_tx:newhashmap,
        }
    }
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    // input transaction and keypair
    // serialize the transaction into vec->string->&[u8]
    let bytes_transaction = bincode::serialize(&t).unwrap();
    let sig = key.sign(bytes_transaction.as_ref());
    // output a signature
    // let bytesig = sig.as_ref();
    return sig;
}

pub fn conversion(public_key: &<Ed25519KeyPair as KeyPair>::PublicKey) -> [u8;20]{
    // hash the public key
    let address = ring::digest::digest(&ring::digest::SHA256, &public_key.as_ref());
    let mut raw_hash: [u8; 20] = [0; 20];
    let a = address.as_ref();
    let num = a.len();
    // take the last 20 bytes
    raw_hash[0..20].copy_from_slice(&a[(num-20)..num]);
    return raw_hash;
}

// // Verify digital signature of a transaction, using public key instead of secret key
// pub fn verify(t: &Transaction, public_key: &<Ed25519KeyPair as KeyPair>::PublicKey, signature: &Signature) -> bool {
//     // use the public key to verify
//     let bytes_transaction = bincode::serialize(&t).unwrap();
//     // let bytes = String::from_utf8(bytes_transaction).unwrap();//
//     let peer_public_key_bytes = public_key.as_ref();
//     let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, peer_public_key_bytes);
//     if let Ok(result) = peer_public_key.verify(bytes_transaction.as_ref(), signature.as_ref()){
//         return true;
//     }
//     else{
//         return false;
//     }
//     // output a bool
// }

pub fn verify(t: &Transaction, public_key: Vec<u8>, signature: Vec<u8>) -> bool {
    // use the public key to verify
    let bytes_transaction = bincode::serialize(&t).unwrap();
    let peer_public_key_bytes = &public_key;
    let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, peer_public_key_bytes);
    // output a bool
    if let Ok(result) = peer_public_key.verify(bytes_transaction.as_ref(), &signature){
        return true;
    }
    else{
        return false;
    }
    
}

// transaction generator
pub struct Context {
    /// Channel for receiving control signal
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
}

pub fn new(
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<Mempool>>,
) -> Context {
    Context {
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
        mempool: Arc::clone(mempool)
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("tx_generator".to_string())
            .spawn(move || {
                self.tx_loop();
            })
            .unwrap();
        info!("tx loop start!");
    }

    fn tx_loop(&mut self) {
        // connect to server to broadcast
        let mut newtxhashes = Vec::new();
        let mut account = Vec::new();

        // initiate diffrent key pair for different node
        for user in 1..3{
            let u = key_pair::random();
            let public_key = u.public_key();
            let account_address = conversion(public_key).into();
            let balance = 50;
            let account_nonce = user;
            account.push(u);
            let mut blc = self.blockchain.lock().unwrap();
            // insert the state into blockchain
            blc.state.insert(account_address, (account_nonce, balance));
            drop(blc);
        }

        loop{
            // generate random keypair
            let send = key_pair::random();
            let receive = key_pair::random();
            let public_key = receive.public_key();

            // let blc = self.blockchain.lock().unwrap();
            // // insert the state into blockchain
            // let st = &blc.state;
            // drop(blc);

            let mut rng = rand::thread_rng();

            // choose receive account
            let blc = self.blockchain.lock().unwrap();
            let mut num = 0;
            let mut recipient_address:H160 = [0;20].into();
            let target_receive = rng.gen_range(0, &blc.state.len());
            for (receiver,x) in &blc.state{
                if num == target_receive{
                    recipient_address = receiver.clone();
                    break;
                }
                else{
                    num += 1;
                }
            }
            // choose send account 
            let account_num = account.len();
            let chosen_send = rng.gen_range(0, account_num);
            let send = conversion(&account[chosen_send].public_key()).into();
            let (an,b) = blc.state.get(&send).expect("failed");
            
            
            // choose send value
            let mut value = 1;
            // if b > &0 {
            //     value = rng.gen_range(1, b);
            // }
            // else{
            //     continue;
            // }

            // set nonce
            let account_nonce = an + 1;
            drop(blc);

            let new_tx = Transaction::new(recipient_address, value, account_nonce.into());
            let signature = sign(&new_tx, &account[chosen_send]).as_ref().to_vec();
            let signed_tx = SignedTransaction::new(new_tx, signature, conversion(account[chosen_send].public_key()).to_vec());
            let newtxhash = signed_tx.hash();
            newtxhashes.push(newtxhash);
            
            //get the lock
            let mut mp = self.mempool.lock().unwrap();
            // save in memepool
            mp.valid_tx.insert(newtxhash.clone(),signed_tx.clone());
            drop(mp);

            // we need to broadcast to worker first,then worker check and add it into mempool
            self.server.broadcast(Message::NewTransactionHashes(newtxhashes.clone()));
            info!("new transaction occured!");//test
            
            // thread sleep
            let interval = time::Duration::from_micros(10000000 as u64);
            thread::sleep(interval);
        }

    }
}

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::crypto::key_pair;

    // pub fn generate_random_transaction() -> Transaction {
    //     // Default::default();
    //     let mut rng = rand::thread_rng();
    //     let (rand_x, rand_y) = rng.gen();
    //     let x = rand_x;
    //     let y = rand_y;
    //     let trans = Transaction{x, y};//??
    //     return trans;
    // }

    // #[test]
    // fn sign_verify() {
    //     let t = generate_random_transaction();
    //     let key = key_pair::random();
    //     let signature = sign(&t, &key);
    //     assert!(verify(&t, &(key.public_key()), &signature));
    // }
} 
