use crate::{Error, Precompile, PrecompileResult, PrecompileWithAddress};
use revm_primitives::Bytes;
use blst::{min_sig::PublicKey, BLST_ERROR};

pub const BLS_SIGNATURE_VALIDATION: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(102),
    Precompile::Standard(bls_signature_validation_run),
);

const BLS_MSG_HASH_LENGTH :u64 = 32;
const BLS_SIGNATURE_LENGTH :u64 = 96;
const BLS_SINGLE_PUBKEY_LENGTH :u64 = 48;
const DST: [u8; 43] = *b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";


/// Run bls signature validation precompile.
///
/// The input is encoded as follows:
/// | msg_hash |  signature  |  [{bls pubkey}]  |
/// |    32    |      96     |      [{48}]      |
fn bls_signature_validation_run(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    let cost = calc_gas_cost(input);
    if cost > gas_limit {
        return Err(Error::OutOfGas);
    }

    let msg_and_sig_length = BLS_MSG_HASH_LENGTH + BLS_SIGNATURE_LENGTH;
    let input_length = input.len() as u64;
    if (input_length<=msg_and_sig_length) || ((input_length-msg_and_sig_length)%BLS_SINGLE_PUBKEY_LENGTH!=0) {
        return Err(Error::BLSInvalidInputLength);
    }

    let msg_hash: &Vec<u8> = &input[..BLS_MSG_HASH_LENGTH as usize].to_vec();
    let signature_data = &input[BLS_MSG_HASH_LENGTH as usize..msg_and_sig_length as usize].to_vec();
    let pub_keys_data = &input[msg_and_sig_length as usize..].to_vec();

    let signature = match blst::min_sig::Signature::from_bytes(signature_data) {
        Ok(sig) => sig,
        Err(_) => return Err(Error::BLSInvalidSignature),
    };

    let pub_key_count = (input_length - msg_and_sig_length) / BLS_SINGLE_PUBKEY_LENGTH;
    let mut pub_keys: Vec<PublicKey> = Vec::new();
    for i in 0..pub_key_count {
        let pub_key = &pub_keys_data[i as usize * BLS_SINGLE_PUBKEY_LENGTH as usize..(i + 1) as usize * BLS_SINGLE_PUBKEY_LENGTH as usize];
        let pk = match blst::min_sig::PublicKey::from_bytes(pub_key) {
            Ok(pk) => pk.clone(),
            Err(_) => return Err(Error::BLSInvalidPublicKey),
        };
        pub_keys.push(pk);
    }
    if pub_keys.is_empty() {
        return Err(Error::BLSInvalidPublicKey);
    }

    if pub_keys.len() == 1 {
        match signature.verify(true, &msg_hash, &DST, &[], &pub_keys[0], true) {
            BLST_ERROR::BLST_SUCCESS => (),
            _ => return Err(Error::BLSInvalidSignature),
        }
    } else {
        let pub_keys_iter = pub_keys.iter();
        let pub_keys_refs: Vec<&PublicKey> = pub_keys_iter.map(|pk| pk).collect();
        match signature.fast_aggregate_verify(true, &msg_hash, &DST,  &pub_keys_refs) {
            BLST_ERROR::BLST_SUCCESS => (),
            _ => return Err(Error::BLSInvalidSignature),
        }
    }

    let output = Bytes::from(vec![1]);
    Ok((cost, output))
}

fn calc_gas_cost(input: &Bytes) -> u64 {
    const BLS_SIGNATURE_VALIDATION_BASE: u64 = 3_000;
    const BLS_SIGNATURE_VALIDATION_PER_KER: u64 = 3_500;

    let msg_length = BLS_MSG_HASH_LENGTH + BLS_SIGNATURE_LENGTH;
    let single_pubkey_length = BLS_SINGLE_PUBKEY_LENGTH;
    let input_length = input.len() as u64;

    if (input_length <= msg_length) || ((input_length - msg_length) % single_pubkey_length != 0) {
        return BLS_SIGNATURE_VALIDATION_BASE;
    }

    let pub_key_number = (input_length - msg_length) / single_pubkey_length;
    let cost = BLS_SIGNATURE_VALIDATION_BASE + BLS_SIGNATURE_VALIDATION_PER_KER * pub_key_number;

    return cost;
}