
use curve25519_dalek::constants;
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use std::cmp::Ordering;
use super::*;


#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;


#[derive(Debug)]
#[allow(non_snake_case)]
pub struct PArray{
    pub a:Scalar,
    pub P:RistrettoPoint
}
#[allow(non_snake_case)]
impl PArray {
    pub fn new(a: Scalar, P: RistrettoPoint) -> Self {
        PArray {
            a,
            P
        }
    }
}

#[allow(unused_comparisons)]
fn compare(a:&Scalar, b:&Scalar) -> Ordering{
    if a.eq(b){
        return Ordering::Equal
    }
    let mut i = 31;
    while i >= 0{
        if a.to_bytes()[i]>b.to_bytes()[i]{
            return Ordering::Greater
        }else if a.to_bytes()[i]<b.to_bytes()[i] {
            return Ordering::Less
        }
        i -= 1;
    }
    Ordering::Equal
}

impl Ord for PArray {
    fn cmp(&self, other: &Self) -> Ordering {
        compare(&self.a, &other.a)
    }
}

impl PartialOrd for PArray {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PArray {
    fn eq(&self, other: &Self) -> bool {
        self.a.eq(&other.a)
    }
}
impl Eq for PArray {}

/// Bos-Coster algorithm
#[allow(non_snake_case)]
pub fn verify_batch_equation_Bos(
    bs: Scalar,
    zs: Vec<Scalar>,
    mut hrams: Vec<Scalar>,
    signatures: &[impl HasR],
    public_keys: &[PublicKey],
    deduplicate_public_keys: bool,
) -> SignatureResult<()>
{
    // let B = once(Some(constants::RISTRETTO_BASEPOINT_POINT));
    let R =signatures.iter().map(|sig| sig.get_R().decompress().unwrap());
    let mut ppks = Vec::new();
    let As = if ! deduplicate_public_keys {
        // Multiply each H(R || A || M) by the random value
        for (hram, z) in hrams.iter_mut().zip(zs.iter()) {
            *hram = &*hram * z;
        }
        public_keys
    } else {
        // TODO: Actually deduplicate all if deduplicate_public_keys is set?
        ppks.reserve( public_keys.len() );
        // Multiply each H(R || A || M) by the random value
        for i in 0..public_keys.len() {
            let zhram = &hrams[i] * zs[i];
            let j = ppks.len().checked_sub(1);
            if j.is_none() || ppks[j.unwrap()] != public_keys[i] {
                ppks.push(public_keys[i]);
                hrams[ppks.len()-1] = zhram;
            } else {
                hrams[ppks.len()-1] = &hrams[ppks.len()-1] + zhram;
            }
        }
        hrams.truncate(ppks.len());
        ppks.as_slice()
    }.iter().map(|pk| Some(pk.as_point().clone()));
    // Compute (-∑ z[i]s[i] (mod l)) B + ∑ z[i]R[i] + ∑ (z[i]H(R||A||M)[i] (mod l)) A[i] = 0
    let left=(&bs * &constants::RISTRETTO_BASEPOINT_TABLE).compress();
    let mut parray:Vec<PArray> = Vec::new();
    for (r,z) in R.zip(zs.iter()){
        parray.push(PArray::new(*z,r));
    }
    for (A,hram) in As.zip(hrams.iter()){
        parray.push(PArray::new(*hram,A.unwrap()));
    }
    // let mut right:RistrettoPoint=parray[0].a * &parray[0].P; 
    // for i in 1..parray.len(){
    //     right= right + parray[i].a * &parray[i].P;
    // }
    let right = boscoster(parray);
    // We need not return SignatureError::PointDecompressionError because
    // the decompression failures occur for R represent invalid signatures.
    let mut b = false;
    if left.eq(&right){b=true;}
    if b { Ok(()) } else { Err(SignatureError::EquationFalse) }
}

#[allow(non_snake_case)]
fn boscoster(mut P: Vec<PArray>)->CompressedRistretto{
    P.sort_by(|a,b|a.cmp(b).reverse());
    while !P[1].a.eq(&Scalar::zero()){
        while P[0].cmp(&P[1]) != Ordering::Less{
        P[0].a=P[0].a - P[1].a;
        P[1].P=P[0].P + P[1].P;
        }
        let timer1 = howlong::HighResolutionTimer::new();
        P.sort_by(|a,b|a.cmp(b).reverse());
        println!("sort used: {:#?}", timer1.elapsed());
    }
    return (P[0].a * &P[0].P).compress();
}