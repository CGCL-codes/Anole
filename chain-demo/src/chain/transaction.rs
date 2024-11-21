use std::sync::atomic::{AtomicU64, Ordering};

use super::*;
use serde::{Deserialize, Serialize};

static TX_ID_CNT: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionValue {
    pub trans_in: bool,
    pub trans_value: TxType,
    pub time_stamp: TsType,
}

impl TransactionValue{
    pub fn to_string(&self) -> String{
        let result =String::from(self.trans_in.to_string())
        + &String::from(self.trans_value.to_string())
        + &String::from(self.time_stamp.to_string());
        result
    }
}
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct RawTransaction {
    pub block_id: IdType,
    pub key: KeyType,
    pub value: TransactionValue,
}

impl RawTransaction{
    fn sign<'a>(&self, key_pair:&'a Keypair) -> Signature {
        let ctx = signing_context(b"");
        let raw_data = String::from(&self.block_id.to_string())
            + &String::from(&self.key)
            + &String::from(&self.value.to_string());
        
        key_pair.sign(ctx.bytes(raw_data.as_bytes()))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: IdType,
    pub block_id: IdType,
    pub key: KeyType,
    pub value: TransactionValue,
    pub signature: SnType,
}

impl Transaction {
    pub fn create_with_sk(tx: &RawTransaction, key_pair: &Keypair) -> Self {
        let id = TX_ID_CNT.fetch_add(1, Ordering::SeqCst) as IdType;
        let block_id = tx.block_id.clone();
        // let signature = sign_transaction();
        let signature = tx.sign(key_pair);
        let key = tx.key.clone();
        let value = tx.value.clone();
        Self {
            id,
            block_id,
            key,
            value,
            signature
        }
    }
}


// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_transaction_create() {
//         let raw_value = TransactionValue {
//             trans_in: true,
//             trans_value: 122,
//             time_stamp: 821,
//         };
//         let raw_transaction = RawTransaction{
//             block_id: 1,
//             key: String::from("76a914a57414e5ffae9ef5074bacbe10a320bb2614e1f388ac"),
//             value: raw_value.clone(),
//         };
//         let tx = Transaction::create(&raw_transaction);
//         assert_eq!(tx.signature,String::from("need to complete"));
//         let expected = Transaction {
//             id: 0,
//             block_id: 1,
//             key: String::from("76a914a57414e5ffae9ef5074bacbe10a320bb2614e1f388ac"),
//             value: raw_value,
//             signature: String::from("need to complete"),
//         };
//         assert_eq!(tx,expected);
//     }
// }
