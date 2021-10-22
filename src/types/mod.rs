mod account;
mod block;
mod blockchain;
mod chain;
mod transaction;

pub use account::{Account, AccountType};
pub use block::Block;
pub use blockchain::Blockchain;
pub use chain::Chain;
pub use transaction::{Transaction, TransactionData};

use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};

pub type Hash = String;
pub type Timestamp = u128;
pub type AccountId = String;
pub type Balance = u128;
pub type Error = String;
pub type PublicKeyBytes = [u8; PUBLIC_KEY_LENGTH];
pub type SecretKeyBytes = [u8; SECRET_KEY_LENGTH];
pub type SignatureBytes = [u8; SIGNATURE_LENGTH];
