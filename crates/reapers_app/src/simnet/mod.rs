//! The chain, rehearsed on the device: a burner wallet, a ledger of real
//! signed transactions, a slip for each as it lands, and an explorer. See
//! `docs/solana.md`, "The simnet".

mod backend;
mod chain;
mod surfpool;

pub use backend::SimnetBackend;
pub use chain::SimnetAccount;
pub use chain::SimnetChain;
pub use chain::SimnetInstruction;
pub use chain::SimnetPost;
pub use chain::SimnetRole;
pub use chain::SimnetTransaction;
pub use surfpool::FaucetReceipt;
pub use surfpool::SignatureStatus;
pub use surfpool::SurfpoolError;
pub use surfpool::SurfpoolFailure;
pub use surfpool::SurfpoolFaucet;
pub use surfpool::SurfpoolResult;
pub use surfpool::SurfpoolRpc;
