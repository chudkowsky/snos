use blockifier::state::cached_state::CachedState;
use cairo_vm::Felt252;
use starknet_os_types::hash::Hash;

use crate::execution::helper::ContractStorageMap;
use crate::starknet::business_logic::fact_state::contract_state_objects::ContractState;
use crate::starknet::business_logic::fact_state::state::SharedState;
use crate::starknet::starknet_storage::OsSingleStarknetStorage;
use crate::starkware_utils::commitment_tree::binary_fact_tree::BinaryFactTree;
use crate::starkware_utils::commitment_tree::errors::TreeError;
use crate::starkware_utils::commitment_tree::leaf_fact::LeafFact;
use crate::starkware_utils::serializable::{DeserializeError, Serializable, SerializationPrefix, SerializeError};
use crate::storage::storage::{DbObject, Fact, HashFunctionType, Storage};

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleLeafFact {
    pub value: Felt252,
}

impl SimpleLeafFact {
    pub fn new(value: Felt252) -> Self {
        Self { value }
    }

    pub fn empty() -> Self {
        Self::new(Felt252::ZERO)
    }
}

impl SerializationPrefix for SimpleLeafFact {}

impl<S, H> Fact<S, H> for SimpleLeafFact
where
    H: HashFunctionType,
    S: Storage,
{
    fn hash(&self) -> Hash {
        Hash::from_bytes_be_slice(&self.serialize().unwrap())
    }
}

impl DbObject for SimpleLeafFact {}

impl Serializable for SimpleLeafFact {
    fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        Ok(self.value.to_bytes_be().to_vec())
    }

    fn deserialize(data: &[u8]) -> Result<Self, DeserializeError> {
        let value = Felt252::from_bytes_be_slice(data);
        Ok(Self { value })
    }
}

impl<S, H> LeafFact<S, H> for SimpleLeafFact
where
    S: Storage,
    H: HashFunctionType,
{
    fn is_empty(&self) -> bool {
        self.value == Felt252::ZERO
    }
}

// TODO: move this function to where it is used, this should not be a public function of
//       the starknet-os library
pub async fn unpack_blockifier_state_async<S: Storage + Send + Sync, H: HashFunctionType + Send + Sync>(
    mut blockifier_state: CachedState<SharedState<S, H>>,
) -> Result<(SharedState<S, H>, SharedState<S, H>), TreeError> {
    let final_state = {
        let state = blockifier_state.state.clone();
        state
            .apply_commitment_state_diff(blockifier_state.to_state_diff().expect("failed to build state diff").into())
            .await?
    };

    let initial_state = blockifier_state.state;

    Ok((initial_state, final_state))
}

/// Translates the (final) Blockifier state into an OS-compatible structure.
///
/// This function uses the fact that `CachedState` is a wrapper around a read-only `DictStateReader`
/// object. The initial state is obtained through this read-only view while the final storage
/// is obtained by extracting the state diff from the `CachedState` part.
pub async fn build_starknet_storage_async<S: Storage + Send + Sync, H: HashFunctionType + Send + Sync>(
    blockifier_state: CachedState<SharedState<S, H>>,
) -> Result<(ContractStorageMap<OsSingleStarknetStorage<S, H>>, SharedState<S, H>, SharedState<S, H>), TreeError> {
    let mut storage_by_address = ContractStorageMap::new();

    // TODO: would be cleaner if `get_leaf()` took &ffc instead of &mut ffc
    let (mut initial_state, mut final_state) = unpack_blockifier_state_async(blockifier_state).await?;

    let all_contracts = final_state.contract_addresses();

    for contract_address in all_contracts {
        let initial_contract_state: ContractState = initial_state
            .contract_states
            .get_leaf(&mut initial_state.ffc, contract_address.clone())
            .await?
            .expect("There should be an initial state");
        let final_contract_state: ContractState = final_state
            .contract_states
            .get_leaf(&mut final_state.ffc, contract_address.clone())
            .await?
            .expect("There should be a final state");

        let initial_tree = initial_contract_state.storage_commitment_tree;
        let updated_tree = final_contract_state.storage_commitment_tree;

        let contract_storage =
            OsSingleStarknetStorage::new(initial_tree, updated_tree, &[], final_state.ffc.clone()).await.unwrap();
        storage_by_address.insert(Felt252::from(contract_address), contract_storage);
    }

    Ok((storage_by_address, initial_state, final_state))
}
