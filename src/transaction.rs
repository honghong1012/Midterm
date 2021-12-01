use serde::{Serialize,Deserialize};
use ring::signature::{self,Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use rand::{Rng};
use crate::crypto::hash::{H160,H256};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    // put anyting in the transaction
    // pub x: i32,
    // pub y: i32,
    pub recipient_address:H160,
    pub value:i32, 
    pub account_nonce:i32,
}

pub struct SignedTansaction {
    // put the transaction and signature together
    pub tx:Transaction,
    pub signature:u8,//?
    pub public_key:u8,//?
}

pub struct Mempool {
    pub valid_tx: HashMap<H256,SignedTansaction>,
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

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &<Ed25519KeyPair as KeyPair>::PublicKey, signature: &Signature) -> bool {
    // use the public key to verify
    let bytes_transaction = bincode::serialize(&t).unwrap();
    // let bytes = String::from_utf8(bytes_transaction).unwrap();//
    let peer_public_key_bytes = public_key.as_ref();
    let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, peer_public_key_bytes);
    if let Ok(result) = peer_public_key.verify(bytes_transaction.as_ref(), signature.as_ref()){
        return true;
    }
    else{
        return false;
    }
    // output a bool
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
