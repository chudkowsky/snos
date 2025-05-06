use cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm::Felt252;
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use super::output::{deserialize_os_state_diff, get_output_info, get_raw_output, read_segment, OsStateDiff};
use crate::error::SnOsError;

const USE_KZG_DA_OFFSET: usize = 0;
const FULL_OUTPUT_OFFSET: usize = 1;
const MERKLE_TREE_ROOT_OFFSET: usize = 2;
const HEADER_SIZE: usize = 3;
const KZG_N_BLOBS_OFFSET: usize = 1;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ShardOsOutput {
    /// Whether KZG data availability was used.
    pub use_kzg_da: Felt252,
    /// Indicates whether previous state values are included in the state update information.
    pub full_output: Felt252,
    /// The merkle tree root of the state diff.
    pub merkle_tree_root: Felt252,
    /// The state diff.
    pub state_diff: Option<OsStateDiff>,
}

impl ShardOsOutput {
    pub fn from_run(vm: &VirtualMachine) -> Result<Self, SnOsError> {
        let (output_base, output_size) = get_output_info(vm)?;
        let raw_output = get_raw_output(vm, output_base, output_size)?;
        deserialize_shard_output(&mut raw_output.into_iter())
    }
}

// Reverse of serialize_os_output in os/output.cairo
pub fn deserialize_shard_output<I>(output_iter: &mut I) -> Result<ShardOsOutput, SnOsError>
where
    I: Iterator<Item = Felt252>,
{
    let header = read_segment(output_iter, HEADER_SIZE, "header elements")?;
    let use_kzg_da = header[USE_KZG_DA_OFFSET];
    let full_output = header[FULL_OUTPUT_OFFSET];
    let merkle_tree_root = header[MERKLE_TREE_ROOT_OFFSET];
    if !use_kzg_da.is_zero() {
        // Skip KZG data.
        let kzg_segment: Vec<_> = output_iter.by_ref().take(2).collect();
        let n_blobs: usize = kzg_segment
            .get(KZG_N_BLOBS_OFFSET)
            .expect("Should have n_blobs in header when using kzg da")
            .to_biguint()
            .try_into()
            .expect("n_blobs should fit in a usize");
        // Skip 'n_blobs' commitments and evaluations.
        let _: Vec<_> = output_iter.by_ref().take(2 * 2 * n_blobs).collect();
    }

    let state_diff =
        if use_kzg_da == Felt252::ZERO { deserialize_os_state_diff(output_iter, full_output)? } else { None };

    Ok(ShardOsOutput { use_kzg_da, full_output, merkle_tree_root, state_diff })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::io::output::ContractChanges;

    #[test]
    /// Tests that the OS output can be serialized and deserialized properly to JSON.
    fn os_output_serde_json() {
        let os_output = ShardOsOutput {
            use_kzg_da: Felt252::ONE,
            full_output: Felt252::ZERO,
            merkle_tree_root: Felt252::ZERO,
            state_diff: Some(OsStateDiff {
                contract_changes: vec![ContractChanges {
                    addr: Felt252::ONE,
                    nonce: Felt252::from(100),
                    class_hash: None,
                    storage_changes: HashMap::from([
                        (
                            Felt252::from_hex_unchecked(
                                "0x723973208639b7839ce298f7ffea61e3f9533872defd7abdb91023db4658812",
                            ),
                            Felt252::from_hex_unchecked("0x1f67eee3d0800"),
                        ),
                        (
                            Felt252::from_hex_unchecked(
                                "0x27e66af6f5df3e043d32367d68ece7e13645cca1ca9f80dfdaff9013fddf0c5",
                            ),
                            Felt252::from_hex_unchecked("0xddec034b926f800"),
                        ),
                    ]),
                }],
            }),
        };

        let os_output_str = serde_json::to_string(&os_output).expect("OS output serialization failed");
        let deserialized_os_output: ShardOsOutput =
            serde_json::from_str(&os_output_str).expect("OS output deserialization failed");

        assert_eq!(deserialized_os_output, os_output);
    }
}
