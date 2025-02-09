use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};

pub use account::{Account, AccountType};
pub use block::Block;
pub use blockchain::Blockchain;
pub use chain::Chain;
pub use transaction::{Transaction, TransactionData};

mod account;
mod block;
mod blockchain;
mod chain;
mod transaction;

pub type Hash = String;
pub type Timestamp = u64;
pub type AccountId = String;
pub type Balance = u128;
pub type Error = String;
pub type PublicKeyBytes = [u8; PUBLIC_KEY_LENGTH];
pub type SecretKeyBytes = [u8; SECRET_KEY_LENGTH];
pub type SignatureBytes = [u8; SIGNATURE_LENGTH];
pub type Target = String;
pub type Bits = i32;
pub type Difficulty = f32;


pub const MAX_TARGET: Bits = 0x1effffff;
pub const EXPECTED_TIME: i32 = 4;