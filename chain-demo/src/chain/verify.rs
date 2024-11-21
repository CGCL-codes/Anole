use serde::{Serialize, Deserialize};


/// To be improved
/// 
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum InvalidReason {
    InvalidSignature,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct VerifyResult(Vec<InvalidReason>);

impl VerifyResult {
    pub fn add(&mut self, reason: InvalidReason) {
        self.0.push(reason);
    }

    pub fn append(&mut self, mut other: Self) {
        self.0.append(&mut other.0);
    }

    pub fn is_ok(&self) -> bool {
        self.0.is_empty()
    }
}