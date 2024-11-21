use curve25519_dalek::ristretto::{CompressedRistretto};
use curve25519_dalek::scalar::Scalar;
use crate::digest::*;
use crate::batch::{HasR,NotAnRng,verify_batch_equation};

use super::*;
use crate::context::{SigningTranscript};
use std::vec::Vec;

const ASSERT_MESSAGE: &'static str = "The number of messages/transcripts, signatures, and public keys must be equal.";

pub type Message = String;

#[allow(non_snake_case)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggSignature {
    pub bs: Scalar,
    pub Rs: Vec<CompressedRistretto>,
}

impl AggSignature{
    #[allow(non_snake_case)]
    pub fn sign_aggregate(
         messages:&[Message],
         signatures: &[Signature],
         public_keys: &[PublicKey],
    ) -> AggSignature
    {
        assert!(signatures.len() == public_keys.len(), "{}", ASSERT_MESSAGE); 
        let zs = calculate_coefficent(messages, signatures, public_keys);
        let bs: Scalar = signatures.iter()
            .map(|sig| sig.s)
            .zip(zs.iter())
            .map(|(s, a)| a * s)
            .sum();

        let Rs = signatures.iter().map(|sig| sig.R).collect();
        AggSignature { bs, Rs }
    }

    /// Verify a half-aggregated aka prepared batch signature
    #[allow(non_snake_case)]
    pub fn verify<T,I>(
        &self,
        transcripts: I,
        messages: &[Message],
        public_keys: &[PublicKey],
        deduplicate_public_keys: bool,
    ) -> SignatureResult<()>
    where
        T: SigningTranscript,
        I: IntoIterator<Item=T>,
    {
        assert!(self.Rs.len() == public_keys.len(), "{}", ASSERT_MESSAGE);  // Check transcripts length below

        let hrams = prepare_batch(transcripts, self.Rs.as_slice(), public_keys);
        let zs = calculate_coefficent(messages, self.Rs.as_slice(), public_keys);
        verify_batch_equation(
            self.bs,
            zs,
            hrams,
            self.Rs.as_slice(),
            public_keys, deduplicate_public_keys
        )
    }

    pub fn byte_len(&self) -> usize {
        32 + 32 * self.Rs.len()
    }

    #[allow(non_snake_case)]
    pub fn write_bytes(&self, mut bytes: &mut [u8]) {
        assert!(bytes.len() == self.byte_len());        
        let mut place = |s: &[u8]| reserve_mut(&mut bytes,32).copy_from_slice(s);
        let mut bs = self.bs.to_bytes();
        bs[31] |= 128;
        place(&bs[..]);
        for R in self.Rs.iter() {
            place(R.as_bytes());
        }
    }    
}

pub fn reserve_mut<'heap, T>(heap: &mut &'heap mut [T], len: usize) -> &'heap mut [T] {
    let tmp: &'heap mut [T] = ::std::mem::replace(&mut *heap, &mut []);
    let (reserved, tmp) = tmp.split_at_mut(len);
    *heap = tmp;
    reserved
}

pub fn calculate_coefficent(
    messages:&[Message],
    signature:&[impl HasR],
    public_keys:&[PublicKey],
)->Vec<Scalar>{
    let mut state = blake2().to_state();
    for i in 0..signature.len(){
        state.update(&signature[i].get_R().to_bytes());
        state.update(&public_keys[i].to_bytes());
        state.update(&messages[i].as_bytes());
    }
    let mut zs: Vec<Scalar> = Vec::new();
    for i in 0..signature.len(){
        let a:String= i.to_string();
        let mut state1 = state.clone();
        state1.update(&a.as_bytes());
        zs.push(Scalar::from_bytes_mod_order(Digest::from(state.finalize()).0));
    }
    zs
}

#[allow(non_snake_case)]
fn prepare_batch<T,I>(
    transcripts: I,
    signatures: &[impl HasR],
    public_keys: &[PublicKey],
) -> Vec<Scalar>
where
    T: SigningTranscript,
    I: IntoIterator<Item=T>,
{

    let mut transcripts = transcripts.into_iter();
    // Compute H(R || A || M) for each (signature, public_key, message) triplet
    let hrams: Vec<Scalar> = transcripts.by_ref()
        .zip(0..signatures.len())
        .map( |(mut t,i)| {
            let mut d = [0u8; 16];
            t.witness_bytes_rng(b"", &mut d, &[&[]], NotAnRng);  // Could speed this up using ZeroRng
            t.proto_name(b"Schnorr-sig");
            t.commit_point(b"sign:pk",public_keys[i].as_compressed());
            t.commit_point(b"sign:R",signatures[i].get_R());
            t.challenge_scalar(b"sign:c")  // context, message, A/public_key, R=rG
        } ).collect();
    assert!(transcripts.next().is_none(), "{}", ASSERT_MESSAGE);
    assert!(hrams.len() == public_keys.len(), "{}", ASSERT_MESSAGE);

    hrams
}