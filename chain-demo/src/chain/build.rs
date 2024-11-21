use std::collections::{HashMap, BTreeMap};
use log::info;
use crate::Digest;
use super::*;

///
/// 
/// For BlockData
/// --
/// intra-index is sorted & storing the first position of distinguished key.
/// 
/// aggre_signs is a series of aggregated signature based on transaction's unique key.
/// 
/// For BlockHeader
/// --
/// 
pub fn build_block<'a>(
    block_id: IdType,
    pre_hash: Digest,
    raw_txs: impl Iterator<Item = &'a RawTransaction>,
    key_pair: & Keypair,
    chain: &mut (impl ReadInterface + WriteInterface),
) -> Result<(BlockHeader, IdType)> {    
    // let param = chain.get_parameter()?;
    let txs: Vec<Transaction> = raw_txs.map(|rtx: &RawTransaction| Transaction::create_with_sk(rtx, &key_pair)).collect();
    let mut _time_stamp: TsType = Default::default();
    let mut tx_ids: Vec<IdType> = Vec::new();
    _time_stamp = txs[0].value.time_stamp;
    // intra-index & aggre_sign
    let mut intra_b_index: BTreeMap<KeyType, IdType>= BTreeMap::new();
    let intra_index: HashMap<KeyType, IdType>;
    let mut aggre_signs: HashMap<KeyType, Signature> = HashMap::new(); 
    let mut aggre_string_txs: String = String::from("");
    let mut pre_key = txs[0].key.clone();
    let mut aggre_sign: Signature;
    let ctx = signing_context(b"");
    // intra_index
    let mut intra_index_size: IdType = 0;

    for tx in txs{
        chain.write_transaction(tx.clone())?;

        if tx.key.eq(&pre_key) {
            aggre_string_txs += &serde_json::to_string(&tx).unwrap();
        } else {
            aggre_sign = key_pair.sign(ctx.bytes(aggre_string_txs.as_bytes()));
            aggre_signs.entry(pre_key).or_insert(aggre_sign);

            aggre_string_txs.clear();
            aggre_string_txs += &serde_json::to_string(&tx).unwrap();
            pre_key = tx.key.clone();
        }
        
        tx_ids.push(tx.id);
        intra_b_index.entry(tx.key).or_insert(tx.id.clone());
        
    }
    aggre_sign = key_pair.sign(ctx.bytes(aggre_string_txs.as_bytes()));
    aggre_signs.entry(pre_key).or_insert(aggre_sign);
    aggre_string_txs.clear();

    let public_key: PkType = key_pair.public.into_compressed();
    // convert BTreeMap to HashMap
    let keys = intra_b_index.keys();
    let values = intra_b_index.values();
    intra_index= keys.into_iter().map(|x| x.to_owned()).zip(values.into_iter().map(|y| y.to_owned())).collect::<HashMap<_,_>>();
    for (key,_id) in intra_index.iter() {
        // u64 eq 8 B
        intra_index_size += (key.as_bytes().len() + 8) as u64;
    }

    let block_header = BlockHeader{
        block_id,
        pre_hash,
        time_stamp: _time_stamp,
        public_key,
    };

    let block_data = BlockData {
        block_id,
        tx_ids,
        intra_index,
        aggre_signs,
    };

    chain.write_block_header(block_header.clone())?;
    chain.write_block_data(block_data.clone())?;

    Ok((block_header, intra_index_size))
}

pub fn build_inter_index(
    block_headers: Vec<BlockHeader>,
    chain: &mut (impl ReadInterface + WriteInterface)
) -> Result<IdType>{
    info!("build inter index");
    let mut inter_indexs: BTreeMap<TsType, InterIndex> = BTreeMap::new();
    let timestamps: Vec<TsType> = Vec::from_iter(block_headers.iter().map(|header| header.time_stamp.to_owned()));
    let heights: Vec<IdType> = Vec::from_iter(block_headers.iter().map(|header| header.block_id.to_owned()));
    let mut param = chain.get_parameter().unwrap();
    let err_bounds = param.error_bounds as FloatType;
    let mut pre_timestamp = timestamps.first().unwrap().to_owned();
    // init inter_index
    inter_indexs.entry(pre_timestamp)
        .or_insert(InterIndex { start_timestamp: pre_timestamp.clone(), regression_a: 1.0, regression_b: 1.0 });
    
    for block_header in block_headers.iter(){
        let mut inter_index = inter_indexs.get(&pre_timestamp).unwrap().to_owned();
        let point_x = block_header.time_stamp as FloatType;
        let point_y = block_header.block_id as FloatType;
        if is_within_boundary(inter_index.regression_a, inter_index.regression_b, point_x, point_y, err_bounds) {
            continue;
        }else {
            // info!("timestamp {:?}", point_x.clone());
            let start_index: usize = timestamps.binary_search(&pre_timestamp).unwrap();
            let end_index_result = timestamps.binary_search(&block_header.time_stamp);
            let end_index: usize =  match end_index_result {
                Ok(_t) => _t,
                Err(_e) => {
                    panic!("problem encounted with binary search timestamp key {:?}",block_header.time_stamp)
                },
            };
            let (regression_a, regression_b) = linear_regression(&timestamps[start_index..end_index + 1], &heights[start_index..end_index + 1]);
            if is_within_boundary(regression_a, regression_b, point_x, point_y, err_bounds) {
                inter_index.regression_a = regression_a;
                inter_index.regression_b = regression_b;
                // update value
                inter_indexs.insert(pre_timestamp.clone(), inter_index.clone());
                continue;
            }else {
                // start new piecewise linear function
                pre_timestamp = block_header.time_stamp.clone();
                // info!("pre_timestamp {:?}",pre_timestamp);
                inter_indexs.entry(pre_timestamp)
                    .or_insert(InterIndex { start_timestamp: pre_timestamp.clone(), regression_a: 1.0, regression_b: 1.0 });
            }

        }   
    }
    
    let mut inter_index_size: IdType = 0;
    //write inter_indexs && count inter_index_size
    for inter_index in inter_indexs.values() {
        // each InterIndex contains 1 TsType & 2 FloatType Storage size eq 3 * 8 = 24 B
        inter_index_size += 24;
        chain.write_inter_index(inter_index.to_owned())?;
        param.inter_index_timestamps.push(inter_index.start_timestamp);
    }
    chain.set_parameter(param.clone())?;
    Ok(inter_index_size)
}