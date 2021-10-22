use crate::traits::{Hashable, WorldState};
use crate::types::{
    Account, AccountId, AccountType, Balance, Error, Hash, PublicKeyBytes, SignatureBytes,
    Timestamp,
};
use blake2::digest::FixedOutput;
use blake2::{Blake2s, Digest};
use ed25519_dalek::{PublicKey, Signature, Verifier};

#[derive(Debug, Clone)]
pub struct Transaction {
    nonce: u128,
    timestamp: Timestamp,
    from: Option<AccountId>,
    pub(crate) data: TransactionData,
    signature: Option<SignatureBytes>,
}

#[derive(Debug, Clone)]
pub enum TransactionData {
    CreateAccount(AccountId, PublicKeyBytes),
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
            TransactionData::CreateAccount(account_id, public_key) => {
                state.create_account(account_id.clone(), AccountType::User, public_key.clone())
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

                let sender = state
                    .get_account_by_id(from.clone())
                    .ok_or("Invalid sender account.".to_string())?;

                let receiver = state
                    .get_account_by_id(to.to_string())
                    .ok_or("Invalid receiver account.".to_string())?;

                if sender.balance < *amount {
                    return Err("Sender doesn't have enough currency.".to_string());
                }

                if !self.verify(&sender.clone()) {
                    return Err("Signature invalid.".to_string());
                }

                let balance = receiver
                    .balance
                    .checked_add(*amount)
                    .ok_or("Transfer amount overflow.".to_string())?;

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

    pub fn verify(&self, sender: &Account) -> bool {
        match self.signature {
            Some(signature) => {
                let pub_key = PublicKey::from_bytes(sender.public_key.as_ref().clone());
                if pub_key.is_ok() {
                    return pub_key
                        .unwrap()
                        .verify(self.hash().as_bytes(), &Signature::from(signature))
                        .is_ok();
                }
            }
            None => return false,
        }
        false
    }

    pub fn set_sign(&mut self, signature: SignatureBytes) {
        self.signature = Some(signature);
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
