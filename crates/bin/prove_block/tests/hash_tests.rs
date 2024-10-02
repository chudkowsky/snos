use std::collections::HashMap;

use cairo_vm::Felt252;
use rpc_client::pathfinder::proofs::PathfinderClassProof;
use rpc_client::RpcClient;
use rstest::rstest;
use starknet::core::types::BlockId;
use starknet::providers::Provider;
use starknet_os_types::compiled_class::GenericCompiledClass;
use starknet_os_types::deprecated_compiled_class::GenericDeprecatedCompiledClass;
use starknet_os_types::sierra_contract_class::GenericSierraContractClass;
use starknet_types_core::felt::Felt;

#[rstest]
// Contract address 0x41a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf
#[case::correct_hash_computation_0("0x07b3e05f48f0c69e4a65ce5e076a66271a527aff2c34ce1083ec6e1526997a69", 78720)]
// Contract address 0x7a3c142b1ef242f093642604c2ac2259da0efa3a0517715c34a722ba2ecd048
#[case::correct_hash_computation_1("0x5c478ee27f2112411f86f207605b2e2c58cdb647bac0df27f660ef2252359c6", 30000)]
#[ignore = "Requires a running Pathfinder node"]
#[tokio::test(flavor = "multi_thread")]
async fn test_recompute_class_hash(#[case] class_hash_str: String, #[case] block_number: u64) {
    let endpoint = std::env::var("PATHFINDER_RPC_URL").expect("Missing PATHFINDER_RPC_URL in env");
    let class_hash = Felt::from_hex(&class_hash_str).unwrap();
    let block_id = BlockId::Number(block_number);

    let rpc_client = RpcClient::new(&endpoint);
    let contract_class = rpc_client.starknet_rpc().get_class(block_id, class_hash).await.unwrap();

    let compiled_class = if let starknet::core::types::ContractClass::Legacy(legacy_cc) = contract_class {
        let compiled_class = GenericDeprecatedCompiledClass::try_from(legacy_cc).unwrap();
        GenericCompiledClass::Cairo0(compiled_class)
    } else {
        panic!("Test intended to test Legacy contracts");
    };

    let recomputed_class_hash = Felt::from(compiled_class.class_hash().unwrap());

    println!("Class hash: {:#x}", class_hash);
    println!("Recomputed class hash: {:#x}", recomputed_class_hash);

    assert_eq!(class_hash, recomputed_class_hash);
}

#[rstest]
// Contract address 0x41a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf
// #[case::key_not_in_proof("0x05dec330eebf36c8672b60db4a718d44762d3ae6d1333e553197acb47ee5a062", 156538)]
#[case::key_not_in_proof("0x05dec330eebf36c8672b60db4a718d44762d3ae6d1333e553197acb47ee5a062", 56355)]
// #[case::key_not_in_proof("0x05dec330eebf36c8672b60db4a718d44762d3ae6d1333e553197acb47ee5a062", 56350)]
//#[case::key_not_in_proof("0x05dec330eebf36c8672b60db4a718d44762d3ae6d1333e553197acb47ee5a062", 50000)]
// #[case::key_not_in_proof("0x05dec330eebf36c8672b60db4a718d44762d3ae6d1333e553197acb47ee5a062", 8968)]
// #[case::key_not_in_proof("0x07b3e05f48f0c69e4a65ce5e076a66271a527aff2c34ce1083ec6e1526997a69", 156541)]
// #[ignore = "Requires a running Pathfinder node"]
#[tokio::test(flavor = "multi_thread")]
async fn test_key_not_in_proof(#[case] class_hash_str: String, #[case] block_number: u64) {
    let endpoint = std::env::var("PATHFINDER_RPC_URL").expect("Missing PATHFINDER_RPC_URL in env");
    let class_hash = Felt::from_hex(&class_hash_str).unwrap();
    let block_id = BlockId::Number(block_number);

    let rpc_client = RpcClient::new(&endpoint);
    let contract_class = rpc_client.starknet_rpc().get_class(block_id, class_hash).await.unwrap();

    let compiled_class = compile_contract_class(contract_class).unwrap();
    let compiled_class_hash = compiled_class.class_hash().unwrap();

    let recomputed_class_hash = Felt::from(compiled_class.class_hash().unwrap());
    assert_eq!(class_hash, recomputed_class_hash);

    let mut class_hash_to_compiled_class_hash: HashMap<Felt252, Felt252> = HashMap::new();
    class_hash_to_compiled_class_hash.insert(class_hash, compiled_class_hash.into());

    let mut class_proofs: HashMap<Felt252, PathfinderClassProof> =
        HashMap::with_capacity(class_hash_to_compiled_class_hash.len());

    for (class_hash, compiled_class_hash) in class_hash_to_compiled_class_hash.iter() {
        let block_proof = rpc_client.pathfinder_rpc().get_class_proof(block_number, class_hash).await.unwrap();

        // If the contract is declared in this block,
        // there is no point to try to verify this
        if *compiled_class_hash != Felt252::ZERO {
            println!("Try to verify proof");
            block_proof.verify(*class_hash).expect("Could not verify class_proof");
        }
        class_proofs.insert(*class_hash, block_proof);
    }
}

fn compile_contract_class(
    contract_class: starknet::core::types::ContractClass,
) -> Result<GenericCompiledClass, Box<dyn std::error::Error>> {
    let compiled_class = match contract_class {
        starknet::core::types::ContractClass::Sierra(flattened_sierra_cc) => {
            let sierra_class = GenericSierraContractClass::from(flattened_sierra_cc);
            let compiled_class = sierra_class.compile()?;
            GenericCompiledClass::Cairo1(compiled_class)
        }
        starknet::core::types::ContractClass::Legacy(legacy_cc) => {
            let compiled_class = GenericDeprecatedCompiledClass::try_from(legacy_cc)?;
            GenericCompiledClass::Cairo0(compiled_class)
        }
    };

    Ok(compiled_class)
}
