
use anyhow::Result;
use curve25519_dalek::ristretto::CompressedRistretto;
use serde::{Serialize, Deserialize};
use super::*;

pub mod transaction;
pub use transaction::*;

pub mod index;
pub use index::*;

pub mod utils;
pub use utils::*;

pub mod build;
pub use build::*;

pub mod query;
pub use query::*;

pub mod verify;
pub use verify::*;

pub type IdType = u64;
// Timestamp size 4 bytes
pub type TsType = u64; 
// public key size 4 bytes
pub type PkType = CompressedRistretto;
// private key 
// pub type SkType = RsaPrivateKey;
// signature
pub type SnType = Signature;
//key
pub type KeyType = String;
//transaction valßßue
pub type TxType = u64;
// FloatType especially for linear regression
pub type FloatType = f64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub error_bounds: FloatType,
    pub inter_index: bool,
    pub intra_index: bool,
    pub start_block_id: u64,
    pub block_count: u64,
    pub inter_index_timestamps: Vec<TsType>,
}

#[async_trait::async_trait]
pub trait LightNodeInterface {
    async fn lightnode_get_parameter(&self) -> Result<Parameter>;
    async fn lightnode_read_block_header(&self, id: IdType) -> Result<BlockHeader>;
}

pub trait ReadInterface {
    fn get_parameter(&self) -> Result<Parameter>;
    fn read_block_header(&self, id: IdType) -> Result<BlockHeader>;
    fn read_block_data(&self, id: IdType) -> Result<BlockData>;
    fn read_inter_index(&self, timestamp: TsType) -> Result<InterIndex>;
    fn read_inter_indexs(&self) -> Result<Vec<InterIndex>>;
    fn read_transaction(&self, id: IdType) -> Result<Transaction>;
}

pub trait WriteInterface {
    fn set_parameter(&mut self, param: Parameter) -> Result<()>;
    fn write_block_header(&mut self, header: BlockHeader) -> Result<()>;
    fn write_block_data(&mut self, data: BlockData) -> Result<()>;
    fn write_inter_index(&mut self, index: InterIndex) -> Result<()>;
    fn write_transaction(&mut self, tx: Transaction) -> Result<()>;
}

#[cfg(test)]
mod tests;
