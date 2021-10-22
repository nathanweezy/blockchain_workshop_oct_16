use crate::types::{Balance, PublicKeyBytes};

#[derive(Debug, Clone)]
pub enum AccountType {
    User,
    Contract,
}

#[derive(Debug, Clone)]
pub struct Account {
    account_type: AccountType,
    pub(crate) balance: Balance,
    pub(crate) public_key: PublicKeyBytes,
}

impl Account {
    pub fn new(account_type: AccountType, public_key: PublicKeyBytes) -> Self {
        Self {
            account_type,
            balance: 0,
            public_key,
        }
    }
}
