use crate::traits::{Hashable, WorldState};
use crate::types::{AccountId, AccountType, Balance, Error, Hash, Timestamp};
use blake2::digest::FixedOutput;
use blake2::{Blake2s, Digest};

#[derive(Debug, Clone)]
pub struct Transaction {
    nonce: u128,
    timestamp: Timestamp,
    from: Option<AccountId>,
    pub(crate) data: TransactionData,
    signature: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TransactionData {
    CreateAccount(AccountId),
    MintInitialSupply { to: AccountId, amount: Balance },
    Transfer { to: AccountId, amount: Balance },
}

impl Transaction {
    pub fn new(data: TransactionData, from: Option<AccountId>) -> Self {
        Self {
            nonce: 0,
            timestamp: 0,
            from,
            data,
            signature: None,
        }
    }

    pub fn execute<T: WorldState>(&self, state: &mut T, is_genesis: bool) -> Result<(), Error> {
        //TODO Task 2: Implement signature
        match &self.data {
            TransactionData::CreateAccount(account_id) => {
                state.create_account(
                    account_id.clone(),
                    AccountType::User
                )
            }
            TransactionData::MintInitialSupply { to, amount } => {
                if !is_genesis {
                    return Err("Initial supply can be minted only in genesis block.".to_string());
                }
                match state.get_account_by_id_mut(to.clone()) {
                    Some(account) => {
                        account.balance += amount;
                        Ok(())
                    }
                    None => Err("Invalid account.".to_string()),
                }
            }
            // TODO Task 1: Implement transfer transition function
            // 1. Check that receiver and sender accounts exist
            // 2. Check sender balance
            // 3. Change sender/receiver balances and save to state
            // 4. Test
            TransactionData::Transfer { to, amount } => {
                if self.from.is_none() {
                    return Err("Invalid sender account id.".to_string());
                }

                let from = &self.from.as_ref().unwrap().clone();

                if from.eq(to) {
                    return Err("Transfer to yourself.".to_string());
                }

                let sender;
                match state.get_account_by_id(from.clone()) {
                    Some(account) => {
                        sender = account;
                    }
                    None => return Err("Invalid sender account.".to_string()),
                }

                let receiver;
                match state.get_account_by_id(to.to_string()) {
                    Some(account) => {
                        receiver = account;
                    }
                    None => return Err("Invalid receiver account.".to_string()),
                }

                if &sender.balance < amount {
                    return Err("Sender doesn't have enough currency.".to_string());
                }

                let balance;
                match receiver.balance.checked_add(*amount) {
                    Some(_balance) => {
                        balance = _balance;
                    }
                    None => return Err("Transfer amount overflow.".to_string()),
                }

                match state.get_account_by_id_mut(from.clone()) {
                    Some(sender) => {
                        sender.balance -= *amount;
                    }
                    None => return Err("Invalid sender account.".to_string()),
                }

                match state.get_account_by_id_mut(to.to_string()) {
                    Some(receiver) => {
                        receiver.balance = balance;
                    }
                    None => return Err("Invalid receiver account.".to_string()),
                }
                Ok(())
            }
        }
    }
}

impl Hashable for Transaction {
    fn hash(&self) -> Hash {
        let mut hasher = Blake2s::new();

        hasher.update(format!(
            "{:?}",
            (
                self.nonce,
                self.timestamp,
                self.from.clone(),
                self.data.clone()
            )
        ));

        hex::encode(hasher.finalize_fixed())
    }
}
