use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::traits::{Hashable, WorldState};
use crate::types::{
    Account, AccountId, AccountType, Bits, Block, Chain, Difficulty, Error, Hash, MAX_TARGET,
    PublicKeyBytes, Target, Timestamp, Transaction,
};
use crate::utils::{get_bits_from_hash, get_timestamp};

#[derive(Default, Debug)]
pub struct Blockchain {
    blocks: Chain<Block>,
    accounts: HashMap<AccountId, Account>,
    transaction_pool: Vec<Transaction>,
    pub(crate) target: Target,
    difficulty: Difficulty,
    first_block_timestamp: Timestamp,
    last_block_timestamp: Timestamp,
}

impl WorldState for Blockchain {
    fn create_account(
        &mut self,
        account_id: AccountId,
        account_type: AccountType,
        public_key: PublicKeyBytes,
    ) -> Result<(), Error> {
        match self.accounts.entry(account_id.clone()) {
            Entry::Occupied(_) => Err(format!("AccountId already exist: {}", account_id)),
            Entry::Vacant(v) => {
                v.insert(Account::new(account_type, public_key));
                Ok(())
            }
        }
    }

    fn get_account_by_id(&self, account_id: AccountId) -> Option<&Account> {
        self.accounts.get(&account_id)
    }

    fn get_account_by_id_mut(&mut self, account_id: AccountId) -> Option<&mut Account> {
        self.accounts.get_mut(&account_id)
    }
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            target: format!("{:x}", MAX_TARGET),
            difficulty: 1,
            ..Default::default()
        }
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), Error> {
        //TODO Task 3: Implement mining

        if !block.verify() {
            return Err("Block has invalid hash".to_string());
        }
        let is_genesis = self.blocks.len() == 0;

        if block.transactions.len() == 0 {
            return Err("Block has 0 transactions.".to_string());
        }

        let account_backup = self.accounts.clone();
        for tx in &block.transactions {
            let res = tx.execute(self, is_genesis);
            if let Err(error) = res {
                self.accounts = account_backup;
                return Err(format!("Error during tx execution: {}", error));
            }
        }

        // TODO Task 3: Append block only if block.hash < target
        // Adjust difficulty of target each block generation (epoch)
        if !is_genesis {
            self.update_difficulty();
            self.update_target();
            let target = Bits::from_str_radix(&self.target.clone(), 16).unwrap();
            if !(get_bits_from_hash(block.hash.as_ref().unwrap().clone()) < target) {
                return Err("Hash greater than target".to_string());
            }
        }
        if is_genesis {
            self.first_block_timestamp = get_timestamp();
        }
        self.last_block_timestamp = get_timestamp();
        self.blocks.append(block);
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Error> {
        let mut block_num = self.blocks.len();
        let mut prev_block_hash: Option<Hash> = None;

        for block in self.blocks.iter() {
            let is_genesis = block_num == 1;

            if !block.verify() {
                return Err(format!("Block {} has invalid hash", block_num));
            }

            if !is_genesis && block.prev_hash.is_none() {
                return Err(format!("Block {} doesn't have prev_hash", block_num));
            }

            if is_genesis && block.prev_hash.is_some() {
                return Err("Genesis block shouldn't have prev_hash".to_string());
            }

            if block_num != self.blocks.len() {
                if let Some(prev_block_hash) = &prev_block_hash {
                    if prev_block_hash != &block.hash.clone().unwrap() {
                        return Err(format!(
                            "Block {} prev_hash doesn't match Block {} hash",
                            block_num + 1,
                            block_num
                        ));
                    }
                }
            }

            prev_block_hash = block.prev_hash.clone();
            block_num -= 1;
        }

        Ok(())
    }

    pub fn get_last_block_hash(&self) -> Option<Hash> {
        self.blocks.head().map(|block| block.hash())
    }

    pub fn update_difficulty(&mut self) {
        let actual_time = (self.last_block_timestamp.clone() - self.first_block_timestamp.clone()) as i32;
        let expected = (2016 * 10 * 60) as Difficulty;
        self.difficulty = actual_time / expected;
        println!("new difficulty {}", self.difficulty.clone());
    }

    pub fn update_target(&mut self) {
        let current_target = Bits::from_str_radix(&self.target.clone(), 16).unwrap();
        let mut new_target = current_target * self.difficulty;
        new_target = if new_target > MAX_TARGET { MAX_TARGET } else { new_target };
        self.target = format!("{:x}", new_target);
        println!("new target {}", self.target.clone());
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{append_block, append_block_with_tx, create_account_tx, create_transfer_tx, mint_initial_supply};

    use super::*;

    #[test]
    fn test_new() {
        let bc = Blockchain::new();
        assert_eq!(bc.get_last_block_hash(), None);
    }

    #[test]
    fn test_append() {
        let bc = &mut Blockchain::new();
        append_block(bc, 1);
        let block = append_block(bc, 2);

        assert_eq!(bc.get_last_block_hash(), block.hash);
    }

    #[test]
    fn test_create_genesis_block() {
        let bc = &mut Blockchain::new();

        let account = "satoshi".to_string();
        let (_, tx_create_account) = create_account_tx(account.clone());
        let tx_mint_initial_supply = mint_initial_supply(account.clone(), 100_000_000);

        assert!(
            append_block_with_tx(bc, 1, vec![tx_create_account, tx_mint_initial_supply]).is_ok()
        );

        let satoshi = bc.get_account_by_id(account.clone());

        assert!(satoshi.is_some());
        assert_eq!(satoshi.unwrap().balance, 100_000_000);
    }

    #[test]
    fn test_create_genesis_block_fails() {
        let mut bc = Blockchain::new();


        let account = "satoshi".to_string();
        let (_, tx_create_account) = create_account_tx(account.clone());
        let tx_mint_initial_supply = mint_initial_supply(account.clone(), 100_000_000);

        let mut block = Block::new(None);
        block.set_nonce(1);
        block.add_transaction(tx_mint_initial_supply);
        block.add_transaction(tx_create_account);

        assert_eq!(
            bc.append_block(block).err().unwrap(),
            "Error during tx execution: Invalid account.".to_string()
        );
    }

    #[test]
    fn test_state_rollback_works() {
        let mut bc = Blockchain::new();

        let account_satoshi = "satoshi".to_string();
        let (_, tx_create_account) = create_account_tx(account_satoshi.clone());
        let tx_mint_initial_supply = mint_initial_supply(account_satoshi.clone(), 100_000_000);

        let mut block = Block::new(None);
        block.set_nonce(1);
        block.add_transaction(tx_create_account);
        block.add_transaction(tx_mint_initial_supply);

        assert!(bc.append_block(block).is_ok());

        let mut block = Block::new(bc.get_last_block_hash());

        let account_alice = "alice".to_string();
        let (_, tx_create_alice) = create_account_tx(account_alice.clone());

        let account_bob = "bob".to_string();
        let (_, tx_create_bob) = create_account_tx(account_bob.clone());

        block.set_nonce(2);
        block.add_transaction(tx_create_alice);
        block.add_transaction(tx_create_bob.clone());
        block.add_transaction(tx_create_bob);

        assert!(bc.append_block(block).is_err());

        assert!(bc.get_account_by_id("satoshi".to_string()).is_some());
        assert!(bc.get_account_by_id("alice".to_string()).is_none());
        assert!(bc.get_account_by_id("bob".to_string()).is_none());
    }

    #[test]
    fn test_validate() {
        let bc = &mut Blockchain::new();


        let account = "satoshi".to_string();
        let (_, tx_create_account) = create_account_tx(account.clone());
        let tx_mint_initial_supply = mint_initial_supply(account.clone(), 100_000_000);

        assert!(
            append_block_with_tx(bc, 1, vec![tx_create_account, tx_mint_initial_supply]).is_ok()
        );

        append_block(bc, 2);
        append_block(bc, 3);

        assert!(bc.validate().is_ok());

        let mut iter = bc.blocks.iter_mut();
        iter.next();
        iter.next();
        let block = iter.next().unwrap();
        block.transactions[1].data = mint_initial_supply(account.clone(), 100).data;

        assert!(bc.validate().is_err());
    }

    #[test]
    fn test_transfers() {
        let bc = &mut Blockchain::new();

        let account_id_satoshi = "satoshi".to_string();
        let (_, tx_create_satoshi) = create_account_tx(account_id_satoshi.clone());
        let tx_mint_initial_supply = mint_initial_supply(account_id_satoshi.clone(), 100_000_000);

        assert!(
            append_block_with_tx(bc, 1, vec![tx_create_satoshi, tx_mint_initial_supply]).is_ok()
        );

        let account_id_alice = "alice".to_string();
        let (keypair_alice, tx_create_alice) = create_account_tx(account_id_alice.clone());

        let account_id_bob = "bob".to_string();
        let (keypair_bob, tx_create_bob) = create_account_tx(account_id_bob.clone());

        assert!(
            append_block_with_tx(bc, 2, vec![tx_create_alice, tx_create_bob]).is_ok()
        );

        assert!(bc.get_account_by_id("satoshi".to_string()).is_some());
        assert!(bc.get_account_by_id("alice".to_string()).is_some());
        assert!(bc.get_account_by_id("bob".to_string()).is_some());

        let mut tx_tr_from_satoshi_alice = create_transfer_tx(
            account_id_satoshi.clone(),
            account_id_alice.clone(),
            10_000_000,
        );
        tx_tr_from_satoshi_alice.sign(&keypair_alice);

        let mut tx_tr_from_satoshi_to_bob = create_transfer_tx(
            account_id_satoshi.clone(),
            account_id_bob.clone(),
            50_000_000,
        );
        tx_tr_from_satoshi_to_bob.sign(&keypair_bob);

        let mut tx_tr_from_bob_to_sastoshi = create_transfer_tx(
            account_id_bob.clone(),
            account_id_satoshi.clone(),
            30_000_000,
        );
        tx_tr_from_bob_to_sastoshi.sign(&keypair_bob);

        assert!(
            append_block_with_tx(bc, 3, vec![
                tx_tr_from_satoshi_alice,
                tx_tr_from_satoshi_to_bob,
                tx_tr_from_bob_to_sastoshi,
            ]).is_ok()
        );

        let alice_new_account = bc.get_account_by_id("alice".to_string());
        if alice_new_account.is_some() {
            assert_eq!(alice_new_account.unwrap().balance, 10_000_000)
        }

        let bob_new_account = bc.get_account_by_id("bob".to_string());
        if bob_new_account.is_some() {
            assert_eq!(bob_new_account.unwrap().balance, 20_000_000)
        }

        let satpshi_new_account = bc.get_account_by_id("satoshi".to_string());
        if satpshi_new_account.is_some() {
            assert_eq!(satpshi_new_account.unwrap().balance, 70_000_000)
        }
    }

    #[test]
    fn test_transfers_fails() {
        let bc = &mut Blockchain::new();

        let account_id_satoshi = "satoshi".to_string();
        let (keypair_satoshi, tx_create_satoshi) = create_account_tx(account_id_satoshi.clone());
        let tx_mint_initial_supply = mint_initial_supply(account_id_satoshi.clone(), 100_000_000);

        assert!(
            append_block_with_tx(bc, 1, vec![
                tx_create_satoshi,
                tx_mint_initial_supply,
            ]).is_ok()
        );

        let account_id_alice = "alice".to_string();
        let (_, tx_create_alice) = create_account_tx(account_id_alice.clone());


        let account_id_bob = "bob".to_string();
        let (_, tx_create_bob) = create_account_tx(account_id_bob.clone());

        assert!(
            append_block_with_tx(bc, 2, vec![
                tx_create_alice,
                tx_create_bob,
            ]).is_ok()
        );

        for acc_id in vec![account_id_satoshi.clone(), account_id_bob.clone(), account_id_alice.clone()] {
            assert!(bc.get_account_by_id(acc_id.to_string()).is_some());
        }

        let mut tx_tr_self = create_transfer_tx(
            account_id_satoshi.clone(),
            account_id_satoshi.clone(),
            10_000_000,
        );
        tx_tr_self.sign(&keypair_satoshi);

        assert_eq!(
            append_block_with_tx(bc, 2, vec![tx_tr_self]).err().unwrap(),
            "Error during tx execution: Transfer to yourself.".to_string()
        );

        let mut tx_tr_gt_balance = create_transfer_tx(
            account_id_satoshi.clone(),
            account_id_bob.clone(),
            100_000_000_000,
        );
        tx_tr_gt_balance.sign(&keypair_satoshi);

        assert_eq!(
            append_block_with_tx(bc, 3, vec![tx_tr_gt_balance]).err().unwrap(),
            "Error during tx execution: Sender doesn't have enough currency.".to_string()
        );

        let mut tx_tr_from_satoshi_to_invalid = create_transfer_tx(
            account_id_satoshi.clone(),
            "invalid".to_string(),
            1,
        );
        tx_tr_from_satoshi_to_invalid.sign(&keypair_satoshi);

        assert_eq!(
            append_block_with_tx(bc, 4, vec![tx_tr_from_satoshi_to_invalid]).err().unwrap(),
            "Error during tx execution: Invalid receiver account.".to_string()
        );

        let tx_tr_from_invalid_to_satoshi = create_transfer_tx(
            "invalid".to_string(),
            account_id_satoshi.clone(),
            1,
        );

        assert_eq!(
            append_block_with_tx(bc, 4, vec![tx_tr_from_invalid_to_satoshi]).err().unwrap(),
            "Error during tx execution: Invalid sender account.".to_string()
        );
    }

    #[test]
    fn test_transfers_sign() {
        let bc = &mut Blockchain::new();

        let account_id_satoshi = "satoshi".to_string();
        let (keypair_satoshi, tx_create_satoshi) = create_account_tx(account_id_satoshi.clone());
        let tx_mint_initial_supply = mint_initial_supply(account_id_satoshi.clone(), 100_000_000);

        assert!(
            append_block_with_tx(bc, 1, vec![
                tx_create_satoshi,
                tx_mint_initial_supply,
            ]).is_ok()
        );

        let account_id_alice = "alice".to_string();
        let (_, tx_create_alice) = create_account_tx(account_id_alice.clone());


        let account_id_bob = "bob".to_string();
        let (keypair_bob, tx_create_bob) = create_account_tx(account_id_bob.clone());

        assert!(
            append_block_with_tx(bc, 2, vec![
                tx_create_alice,
                tx_create_bob,
            ]).is_ok()
        );

        let mut tx_tr_from_satoshi_to_bob_wtih_fake_sign = create_transfer_tx(
            account_id_satoshi.clone(),
            account_id_bob.clone(),
            1,
        );

        tx_tr_from_satoshi_to_bob_wtih_fake_sign.sign(&keypair_bob);

        assert!(
            append_block_with_tx(bc, 2, vec![
                tx_tr_from_satoshi_to_bob_wtih_fake_sign
            ]).is_err()
        );

        let mut tx_tr_from_satoshi_to_bob_wtih_fake_data = create_transfer_tx(
            account_id_satoshi.clone(),
            account_id_bob.clone(),
            1,
        );
        tx_tr_from_satoshi_to_bob_wtih_fake_data.sign(&keypair_satoshi);

        let tx_fake = create_transfer_tx(
            account_id_satoshi.clone(),
            account_id_bob.clone(),
            500,
        );
        tx_tr_from_satoshi_to_bob_wtih_fake_data.data = tx_fake.data;
        assert!(
            append_block_with_tx(bc, 2, vec![
                tx_tr_from_satoshi_to_bob_wtih_fake_data
            ]).is_err()
        );
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
        dbg!(bc.target.clone());

        // let mut block = Block::new(bc.get_last_block_hash());
        // let keypair_alice = Keypair::generate(&mut rand::rngs::OsRng {});
        // let tx_create_alice = Transaction::new(
        //     TransactionData::CreateAccount(
        //         "alice".to_string(),
        //         keypair_alice.public.as_bytes().clone(),
        //     ),
        //     None,
        // );
        // block.add_transaction(tx_create_alice);
        // // block.mine(bc.target.clone());
        // assert!(bc.append_block(block).is_ok());
        // dbg!(bc.target.clone());
        //
        // let mut block = Block::new(bc.get_last_block_hash());
        // let keypair_bob = Keypair::generate(&mut rand::rngs::OsRng {});
        // let tx_create_bob = Transaction::new(
        //     TransactionData::CreateAccount(
        //         "bob".to_string(),
        //         keypair_bob.public.as_bytes().clone(),
        //     ),
        //     None,
        // );
        //
        // block.add_transaction(tx_create_bob);
        // // block.mine(bc.target.clone());
        // assert!(bc.append_block(block).is_ok());
        // dbg!(bc.target.clone());
        // assert!(bc.get_account_by_id("satoshi".to_string()).is_some());
        // assert!(bc.get_account_by_id("alice".to_string()).is_some());
        // assert!(bc.get_account_by_id("bob".to_string()).is_some());
    }
}
