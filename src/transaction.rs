use serde::{Serialize,Deserialize};
use ring::signature::{self,Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use rand::{Rng};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    // put anyting in the transaction
    pub x: i32,
    pub y: i32
    // input? output?
}

// pub struct SignedTansaction {
//     // put the transaction and signature together
// }

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    // combine the signature together with the transaciton

    // input transaction and keypair
    // serialize the transaction into vec->string->&[u8]
    let bytes_transaction = bincode::serialize(&t).unwrap();
    // let bytes = String::from_utf8(bytes_transaction).unwrap();
    let sig = key.sign(bytes_transaction.as_ref());
    // output a signature
    return sig;
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

    pub fn generate_random_transaction() -> Transaction {
        // Default::default();
        let mut rng = rand::thread_rng();
        let (rand_x, rand_y) = rng.gen();
        let x = rand_x;
        let y = rand_y;
        let trans = Transaction{x, y};
        return trans;
    }

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, &(key.public_key()), &signature));
    }
}
