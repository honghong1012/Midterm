use super::hash::{Hashable, H256};

// Merkle node
#[derive(Debug, Default, Clone)]
pub struct MerkleTreeNode {
    left: Option<Box<MerkleTreeNode>>,
    right: Option<Box<MerkleTreeNode>>,
    // left: Option<Rc<RefCell<MerkleTreeNode>>>,
    // right: Option<Rc<RefCell<MerkleTreeNode>>>,
    hash: H256,
}

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    root: MerkleTreeNode,
    level_count: usize,
}

/// Given the hash of the left and right nodes, compute the hash of the parent node.
fn hash_children(left: &H256, right: &H256) -> H256 {
    // unimplemented!();
    let one = left.as_ref();
    let two = right.as_ref();
    // concat the string
    let new = [one, two].concat();
    // use & to slice the vec<u8>
    let new_f = ring::digest::digest(&ring::digest::SHA256, &new);
    // use into() to transfer the output
    let final_hash = new_f.into();
    return final_hash;
}

/// Duplicate the last node in `nodes` to make its length even.
fn duplicate_last_node(nodes: &mut Vec<Option<MerkleTreeNode>>) {
    // unimplemented!();
    let new_node = nodes.get(0);
    if let Some(v) = new_node.unwrap(){
        let new_hash = v.hash;
        nodes.push(Some(MerkleTreeNode{hash: new_hash, left: None, right: None}));
    }
}


impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        assert!(!data.is_empty());

        // create the leaf nodes:
        let mut curr_level: Vec<Option<MerkleTreeNode>> = Vec::new();
        for item in data {
            curr_level.push(Some(MerkleTreeNode { hash: item.hash(), left: None, right: None }));
        }
        let mut level_count = 1;
        
        // create the upper levels of the tree:
        while curr_level.len() > 1 {
            // Whenever a level of the tree has odd number of nodes, duplicate the last node to make the number even:
            if curr_level.len() % 2 == 1 {
                duplicate_last_node(&mut curr_level); // TODO: implement this helper function
            }
            assert_eq!(curr_level.len() % 2, 0); // make sure we now have even number of nodes.

            let mut next_level: Vec<Option<MerkleTreeNode>> = Vec::new();
            for i in 0..curr_level.len() / 2 {
                let left = curr_level[i * 2].take().unwrap();
                let right = curr_level[i * 2 + 1].take().unwrap();
                let hash = hash_children(&left.hash, &right.hash); // TODO: implement this helper function
                // HAS CHANGED
                // next_level.push(Some(MerkleTreeNode { hash: hash, left: Some(Rc::new(RefCell::new(left))), right: Some(Rc::new(RefCell::new(right))) }));
                next_level.push(Some(MerkleTreeNode { hash: hash, left: Some(Box::new(left)), right: Some(Box::new(right)) }));
            }
            curr_level = next_level;
            level_count += 1;
        }
        MerkleTree {
            root: curr_level[0].take().unwrap(),
            level_count: level_count,
        }
    }

    pub fn root(&self) -> H256 {
        self.root.hash
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let level = self.level_count;
        let node_num = 2^(level - 1);
        let mut real_level = level;
        let mut flag = 0;
        let mut middle = node_num / 2;
        // let node = &self.root;
        let left = self.root.left.clone().unwrap();
        let right = self.root.right.clone().unwrap();
        let mut proof: Vec<H256> = Vec::new();
        let mut leaf = self.root.left.clone().unwrap();
        let mut side_proof = right.clone();
        while real_level > 1 {
            if index < middle {
                if flag == 0{
                    middle = middle - middle / 2;
                    leaf = left.clone();
                    side_proof = right.clone();
                    // proof.push(side_proof.hash);
                }
                if flag == 1{
                    middle = middle - middle / 2;
                    side_proof = leaf.right.clone().unwrap();
                    leaf = leaf.left.clone().unwrap();
                }
                if flag == 2{
                    middle = middle + middle / 2;
                    side_proof = leaf.right.clone().unwrap();
                    leaf = leaf.left.clone().unwrap();
                }
                proof.push(side_proof.hash);
                flag = 1;
            }
            if index >= middle {
                if flag == 0{
                    middle = middle + middle / 2;
                    leaf = left.right.clone().unwrap();
                    side_proof = left.clone();
                }
                if flag == 1{
                    middle = middle - middle / 2;
                    side_proof = leaf.left.clone().unwrap();
                    leaf = leaf.right.clone().unwrap();
                }
                if flag == 2{
                    middle = middle + middle / 2;
                    side_proof = leaf.left.clone().unwrap();
                    leaf = leaf.right.clone().unwrap();
                }
                proof.push(side_proof.hash);
                flag = 2;
            }
            real_level -=1;  
        }
        return proof;

    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    // let real_root = root;
    let cur_proof : Vec<H256>;
    cur_proof = proof.to_vec();
    let proof_num = cur_proof.len();
    let mut iter = 0;
    let mut next_hash = datum.clone();
    println!("{:?}", proof);
    for i in [0..proof_num]{
        next_hash = hash_children(&next_hash, &cur_proof[proof_num - 1 - iter]);
        iter = iter + 1;
    }
    if next_hash == *root{
        return true;
    }
    else{
        return false;
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        //左边是匹配模式 右边是等待展开的代码
        () => {{  // 表示此宏不接受任何参数 会展开这个代码块里的具体内容
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        // println!("{:?}", proof);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }
}
