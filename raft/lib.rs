mod executor;
mod io;
mod log;
mod node;
mod rpc;
mod state;
mod storage;

/// Eletction term
pub type Term = u64;
/// Log Index
pub type Index = u64;
pub type NodeId = u64;
