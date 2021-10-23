use std::ops::Div;
use std::time::{SystemTime, UNIX_EPOCH};

use blake2::{Blake2s, Digest};
use ed25519_dalek::Keypair;
use rand::Rng;

use crate::types::{
    AccountId, Block, Blockchain, Error, Hash, Timestamp, Transaction, TransactionData,
};

pub fn get_bits_from_hash(hash: Hash) -> i32 {
    let mut a = 0;
    for (i, char) in hash.clone().chars().enumerate() {
        if char != '0' {
            a = i;
            break;
        }
    }
    let mut hash = hash[a..hash.len()].to_string().clone();
    while hash[0..6].ends_with("0") {
        hash.insert(0, '0');
    }
    if hash.len() % 2 != 0 {
        hash.insert(0, '0');
    }

    let exponent = format!("{:02x}", hash.len().div(2));
    dbg!(exponent.clone());
    let cofficient = &hash[0..6].to_string().clone();

    let bits = i32::from_str_radix(
        &(exponent.to_string() + cofficient.to_string().as_ref()),
        16,
    )
        .unwrap();
    bits
}

pub fn generate_account_id() -> AccountId {
    let mut rng = rand::thread_rng();
    let seed: u128 = rng.gen();

    hex::encode(Blake2s::digest(&seed.to_be_bytes()))
}

pub fn append_block(bc: &mut Blockchain, nonce: u128) -> Block {
    let mut block = Block::new(bc.get_last_block_hash());
    let keypair_account = Keypair::generate(&mut rand::rngs::OsRng {});
    let tx_create_account = Transaction::new(
        TransactionData::CreateAccount(
            generate_account_id(),
            keypair_account.public.as_bytes().clone(),
        ),
        None,
    );
    block.set_nonce(nonce);
    block.add_transaction(tx_create_account);
    let block_clone = block.clone();

    assert!(bc.append_block(block).is_ok());

    block_clone
}

pub fn append_block_with_tx(
    bc: &mut Blockchain,
    nonce: u128,
    transactions: Vec<Transaction>,
) -> Result<(), Error> {
    let mut block = Block::new(bc.get_last_block_hash());
    block.set_nonce(nonce);

    for tx in transactions {
        block.add_transaction(tx);
    }

    bc.append_block(block)
}

pub fn get_timestamp() -> Timestamp {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH);

    since_the_epoch.unwrap().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        dbg!(generate_account_id());
    }
}
