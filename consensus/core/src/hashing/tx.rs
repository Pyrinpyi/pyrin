use super::HasherExtensions;
use crate::tx::{Transaction, TransactionId, TransactionInput, TransactionOutpoint, TransactionOutput};
use kaspa_hashes::{Hash, Hasher};

/// A bitmask defining which transaction fields we
/// want to encode and which to ignore.
type TxEncodingFlags = u8;

pub const TX_ENCODING_FULL: TxEncodingFlags = 0;
pub const TX_ENCODING_EXCLUDE_SIGNATURE_SCRIPT: TxEncodingFlags = 1;

/// Returns the transaction hash. Note that this is different than the transaction ID.
pub fn hash(tx: &Transaction, include_mass_field: bool) -> Hash {
    let mut hasher = kaspa_hashes::TransactionHash::new();
    write_transaction(&mut hasher, tx, TX_ENCODING_FULL, include_mass_field);
    hasher.finalize()
}

/// Not intended for direct use by clients. Instead use `tx.id()`
pub(crate) fn id(tx: &Transaction) -> TransactionId {
    // Encode the transaction, replace signature script with zeroes, cut off
    // payload and hash the result.

    let encoding_flags = if tx.is_coinbase() { TX_ENCODING_FULL } else { TX_ENCODING_EXCLUDE_SIGNATURE_SCRIPT };
    let mut hasher = kaspa_hashes::TransactionID::new();
    write_transaction(&mut hasher, tx, encoding_flags, false);
    hasher.finalize()
}

/// Write the transaction into the provided hasher according to the encoding flags
fn write_transaction<T: Hasher>(hasher: &mut T, tx: &Transaction, encoding_flags: TxEncodingFlags, include_mass_field: bool) {
    hasher.update(tx.version.to_le_bytes()).write_len(tx.inputs.len());
    for input in tx.inputs.iter() {
        // Write the tx input
        write_input(hasher, input, encoding_flags);
    }

    hasher.write_len(tx.outputs.len());
    for output in tx.outputs.iter() {
        // Write the tx output
        write_output(hasher, output);
    }

    hasher.update(tx.lock_time.to_le_bytes()).update(&tx.subnetwork_id).update(tx.gas.to_le_bytes()).write_var_bytes(&tx.payload);

    // TODO:
    //      1. Avoid passing a boolean and hash the mass only if > 0 (requires setting the mass to 0 on BBT).
    //      2. Use TxEncodingFlags to avoid including the mass for tx ID
    if include_mass_field {
        let mass = tx.mass();
        if mass > 0 {
            hasher.update(mass.to_le_bytes());
        }
    }
}

#[inline(always)]
fn write_input<T: Hasher>(hasher: &mut T, input: &TransactionInput, encoding_flags: TxEncodingFlags) {
    write_outpoint(hasher, &input.previous_outpoint);
    if encoding_flags & TX_ENCODING_EXCLUDE_SIGNATURE_SCRIPT != TX_ENCODING_EXCLUDE_SIGNATURE_SCRIPT {
        hasher.write_var_bytes(input.signature_script.as_slice()).update([input.sig_op_count]);
    } else {
        hasher.write_var_bytes(&[]);
    }
    hasher.update(input.sequence.to_le_bytes());
}

#[inline(always)]
fn write_outpoint<T: Hasher>(hasher: &mut T, outpoint: &TransactionOutpoint) {
    hasher.update(outpoint.transaction_id).update(outpoint.index.to_le_bytes());
}

#[inline(always)]
fn write_output<T: Hasher>(hasher: &mut T, output: &TransactionOutput) {
    hasher
        .update(output.value.to_le_bytes())
        .update(output.script_public_key.version().to_le_bytes())
        .write_var_bytes(output.script_public_key.script());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        subnets::{self, SubnetworkId},
        tx::{scriptvec, ScriptPublicKey},
    };
    use std::str::FromStr;

    #[test]
    fn test_transaction_hashing() {
        struct Test {
            tx: Transaction,
            expected_id: &'static str,
            expected_hash: &'static str,
        }

        let mut tests = vec![
            // Test #1
            Test {
                tx: Transaction::new(0, Vec::new(), Vec::new(), 0, SubnetworkId::from_byte(0), 0, Vec::new()),
                expected_id: "f83b4653b526e7c410dfe83215d97a27ecb180b7ba1508b995c28c3983e4b902",
                expected_hash: "84d819d71bfdb9847593bfbced3632bb4c13a0b8a2b1b82b03eecb8353f32869",
            },
        ];

        let inputs = vec![TransactionInput::new(TransactionOutpoint::new(Hash::from_u64_word(0), 2), vec![1, 2], 7, 5)];

        // Test #2
        tests.push(Test {
            tx: Transaction::new(1, inputs.clone(), Vec::new(), 0, SubnetworkId::from_byte(0), 0, Vec::new()),
            expected_id: "b887b8f6bcde581ed2bea1d8ccc7e1caeed7be2c9db808064dac2b383ee6dec0",
            expected_hash: "7a3bdd484f2ef6abb66a5789d95b23f2e1a5753e7f0eaae3d73586b9df60086a",
        });

        let outputs = vec![TransactionOutput::new(1564, ScriptPublicKey::new(7, scriptvec![1, 2, 3, 4, 5]))];

        // Test #3
        tests.push(Test {
            tx: Transaction::new(1, inputs.clone(), outputs.clone(), 0, SubnetworkId::from_byte(0), 0, Vec::new()),
            expected_id: "5848a1b4843b4c0f6c0a33a9b3486805e72476805f33fad52651ab59deac4f81",
            expected_hash: "5cfc61ff329d40fe62ae383a4cb21857cf592425610e58672c81e291ba4861f2",
        });

        // Test #4
        tests.push(Test {
            tx: Transaction::new(2, inputs, outputs.clone(), 54, SubnetworkId::from_byte(0), 3, Vec::new()),
            expected_id: "b4c796afd46beb1d6a674d21de3ade8c182f997d55bacbc0f681e8ec7219b27d",
            expected_hash: "152ccead0a1e260ab88527b86a80b6ea6e7eb1a5ae8718b8cd8801b3790b1c1f",
        });

        let inputs = vec![TransactionInput::new(
            TransactionOutpoint::new(Hash::from_str("b4c796afd46beb1d6a674d21de3ade8c182f997d55bacbc0f681e8ec7219b27d").unwrap(), 2),
            vec![1, 2],
            7,
            5,
        )];

        // Test #5
        tests.push(Test {
            tx: Transaction::new(2, inputs.clone(), outputs.clone(), 54, SubnetworkId::from_byte(0), 3, Vec::new()),
            expected_id: "c2266b0d084cd36f2cab3c664eec06307c65c9fdb536160544f3ef49082c33e7",
            expected_hash: "11f6c1f568f3be13cf0d459c94db37ab2840afe5bbc628d12d04fae3bdcea136",
        });

        // Test #6
        tests.push(Test {
            tx: Transaction::new(2, inputs.clone(), outputs.clone(), 54, subnets::SUBNETWORK_ID_COINBASE, 3, Vec::new()),
            expected_id: "cbd095375dc9d004e880a9c813e092fcdb5856b512fb23990c0eebf49735956f",
            expected_hash: "f8efa556479c3a559d582be0424f085eeaf8f0f78ffdbd90867b1f9a4a6a99ca",
        });

        // Test #7
        tests.push(Test {
            tx: Transaction::new(2, inputs.clone(), outputs.clone(), 54, subnets::SUBNETWORK_ID_REGISTRY, 3, Vec::new()),
            expected_id: "761a7c9e1aa348e64b7d21fce8afaa2a48f2dfcb545f64bc016b25e1a3582bfe",
            expected_hash: "248942c819e2318807e5ec877ecad14a0b0fa9baf66af0ca30761a63cced0113",
        });

        for (i, test) in tests.iter().enumerate() {
            assert_eq!(test.tx.id(), Hash::from_str(test.expected_id).unwrap(), "transaction id failed for test {}", i + 1);
            assert_eq!(
                hash(&test.tx, false),
                Hash::from_str(test.expected_hash).unwrap(),
                "transaction hash failed for test {}",
                i + 1
            );
        }

        // Avoid compiler warnings on the last clone
        drop(inputs);
        drop(outputs);
    }
}
