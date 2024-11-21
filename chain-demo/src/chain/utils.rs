use anyhow::{Context, Result};
use std::path::Path;
use std::collections::{BTreeMap};
use super::*;
use std::io::BufReader;
use std::fs::File;
use std::io::prelude::*;


/*
input format: block_id sep key sep value
sep = space
key = [address]
value = {in/out, amount, timestamp}
*/
pub fn load_raw_tx_from_file(path: &Path) -> Result<BTreeMap<IdType, Vec<RawTransaction>>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    load_raw_tx_from_str(&buf)

}

pub fn load_raw_tx_from_str(input: &str) -> Result<BTreeMap<IdType, Vec<RawTransaction>>> {
    let mut res = BTreeMap::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty(){
            continue;
        }
        let mut split_str = line.splitn(3, |c| c == '[' || c == ']');
        let block_id: IdType = split_str
            .next()
            .context(format!("failed to parse line {}", line))?
            .trim()
            .parse()?;
        let key: KeyType = split_str
            .next()
            .context(format!("failed to parse line {}", line))?
            .trim()
            .parse()?;
        let raw_value: Vec<String> = split_str
            .next()
            .context(format!("failed to parse line {}", line))?
            .trim()
            .replace('{',"")
            .replace('}',"")
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();
        let mut iter = raw_value.iter();
        let value = TransactionValue {
            trans_in: iter.next().unwrap().eq("in"),
            trans_value: iter.next().unwrap().parse::<TxType>().unwrap(),
            time_stamp: iter.next().unwrap().parse::<TsType>().unwrap(),
        };
        let raw_tx = RawTransaction {
            block_id,
            key,
            value,
        };
        res.entry(block_id).or_insert_with(Vec::new).push(raw_tx);
    }
    Ok(res)
}

/// Linear Regression
/// Default data type FloatType=f64
pub fn linear_regression(arr_x: &[TsType], arr_y: &[IdType]) -> (FloatType, FloatType) {
    let mut sum_x: FloatType = 0.0;
    let mut sum_y: FloatType = 0.0;
    let mut sum_a1: FloatType = 0.0;
    let mut sum_a2: FloatType = 0.0;
    let len_x = arr_x.len();
    for k in 0..len_x {
        sum_x = sum_x + arr_x[k] as FloatType;
        sum_y = sum_y + arr_y[k] as FloatType;
    }
    let len_y: FloatType = len_x as FloatType;
    let avg_x: FloatType = sum_x / len_y;
    let avg_y: FloatType = sum_y / len_y;
    for k in 0..len_x {
        sum_a1 = sum_a1 + (arr_x[k] as FloatType - avg_x) * (arr_y[k] as FloatType - avg_y);
        sum_a2 = sum_a2 + (arr_x[k] as FloatType - avg_x) * (arr_x[k] as FloatType - avg_x);
    }
    let r_a: FloatType = sum_a1 / sum_a2;
    let r_b: FloatType = avg_y - r_a * avg_x;
    (r_a, r_b)
}

/// assume arr is sorted in upper manner, return the lower boundary of input value
/// 
/// input
/// @arr_px
/// @value: t
pub fn variant_binary_search(arr_px: &[TsType], t: TsType) -> TsType {
    let len_px = arr_px.len();
    let mut low = 0;
    let mut high = len_px - 1;
    while low <= high {
        let mid = (low + high)/2;
        if arr_px[mid] >= t {
            if mid == 0 || arr_px[mid+1] < t {
                return arr_px[mid]; 
            } else {
                high = mid - 1;
            }
        } else {
            low = mid + 1;
        }
    }
    arr_px[low - 1]
}

pub fn is_within_boundary (
    a: FloatType,
    b: FloatType, 
    point_x: FloatType, 
    point_y: FloatType, 
    err_bounds: FloatType
) -> bool{
    let y = a * point_x + b;
    FloatType::abs(point_y - y) < err_bounds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_str() {
        let input = "1 [muhtvdmsnbQEPFuEmxcChX58fGvXaaUoVt] {in, 50, 1571443461}\n 1 [mvbnrCX3bg1cDRUu8pkecrvP6vQkSLDSou] {out, 10, 1571443461}";
        let expect = {
            let mut out: BTreeMap<IdType, Vec<RawTransaction>> = BTreeMap::new();
            out.insert(
                1,
                vec![
                    RawTransaction {
                        block_id: 1,
                        key: String::from("muhtvdmsnbQEPFuEmxcChX58fGvXaaUoVt"),
                        value: TransactionValue {
                            trans_in: true,
                            trans_value: 50,
                            time_stamp: 1571443461,
                        }
                    },
                    RawTransaction {
                        block_id: 1,
                        key: String::from("mvbnrCX3bg1cDRUu8pkecrvP6vQkSLDSou"),
                        value: TransactionValue {
                            trans_in: false,
                            trans_value: 10,
                            time_stamp: 1571443461,
                        }
                    },
                ],);
            out
        };
        assert_eq!(load_raw_tx_from_str(&input).unwrap(),expect);
    }
}