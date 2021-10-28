use blake2::{Blake2s, Digest};
use blake2::digest::FixedOutput;

use crate::traits::Hashable;
use crate::types::{Bits, Hash, Target, Timestamp, Transaction};
use crate::utils::{get_bits_from_hash, get_timestamp};

#[derive(Default, Debug, Clone)]
pub struct Block {
    nonce: u128,
    timestamp: Timestamp,
    pub(crate) hash: Option<Hash>,
    pub(crate) prev_hash: Option<Hash>,
    pub(crate) transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(prev_hash: Option<Hash>) -> Self {
        let mut block = Block {
            prev_hash,
            timestamp: get_timestamp(),
            ..Default::default()
        };
        block.update_hash();
        block
    }

    pub fn set_nonce(&mut self, nonce: u128) {
        self.nonce = nonce;
        self.update_hash();
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
        self.update_hash();
    }

    pub fn verify(&self) -> bool {
        matches!(&self.hash, Some(hash) if hash == &self.hash())
    }

    fn update_hash(&mut self) {
        self.hash = Some(self.hash());
    }

    pub fn mine(&mut self, target: Target) {
        let mut nonce = 1;
        let target = Bits::from_str_radix(&target.clone(), 16).unwrap();
        while !(get_bits_from_hash(self.hash.as_ref().unwrap().clone()) < target) {
            nonce += 1;
            self.set_nonce(nonce);
            // println!("{} {} {} bits {}", nonce, &self.hash.as_ref().unwrap().clone(), get_bits_from_hash(self.hash.as_ref().unwrap().clone()), format!("{:2x}", get_bits_from_hash(self.hash.as_ref().unwrap().clone())));
        }
        println!("GOT IT {} {}", nonce, &self.hash.as_ref().unwrap().clone());
    }
}

impl Hashable for Block {
    fn hash(&self) -> Hash {
        let mut hasher = Blake2s::new();
        hasher.update(format!("{:?}", (self.prev_hash.clone(), self.nonce)).as_bytes());
        for tx in self.transactions.iter() {
            hasher.update(tx.hash())
        }

        hex::encode(hasher.finalize_fixed())
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::Keypair;

    use crate::{types::{Blockchain, TransactionData}, utils::{create_account_tx, generate_account_id, mint_initial_supply}};

    use super::*;

    #[test]
    fn test_creation() {
        let mut block = Block::new(None);
        let keypair_account = Keypair::generate(&mut rand::rngs::OsRng {});
        let tx = Transaction::new(
            TransactionData::CreateAccount(
                "alice".to_string(),
                keypair_account.public.as_bytes().clone(),
            ),
            None,
        );
        block.set_nonce(1);
        block.add_transaction(tx);

        dbg!(block);
    }

    #[test]
    fn test_hash() {
        let mut block = Block::new(None);

        let keypair_account = Keypair::generate(&mut rand::rngs::OsRng {});
        let tx = Transaction::new(
            TransactionData::CreateAccount(
                "alice".to_string(),
                keypair_account.public.as_bytes().clone(),
            ),
            None,
        );
        block.set_nonce(1);

        let hash1 = block.hash();

        block.add_transaction(tx);
        block.set_nonce(1);
        let hash2 = block.hash();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_mining() {
        let mut bc = Blockchain::new();

        let account_id_satoshi = "satoshi".to_string();
        let (_, tx_create_satoshi) = create_account_tx(account_id_satoshi.clone());
        let tx_mint_initial_supply = mint_initial_supply(account_id_satoshi.clone(), 100_000_000);

        let mut block = Block::new(bc.get_last_block_hash());
        block.add_transaction(tx_create_satoshi);
        block.add_transaction(tx_mint_initial_supply);
        block.mine(bc.target.clone());
        assert!(bc.append_block(block).is_ok());

        let mut count = 0;
        loop {
            count += 1;
            let mut block = Block::new(bc.get_last_block_hash());
            let (_, tx_create_alice) = create_account_tx(generate_account_id());
            block.add_transaction(tx_create_alice);
            block.mine(bc.target.clone());
            assert!(bc.append_block(block).is_ok());
            if count == 10 {
                break;
            }
        }

    }
}
