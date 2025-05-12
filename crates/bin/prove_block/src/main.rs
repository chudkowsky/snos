use cairo_vm::types::layout_name::LayoutName;
use cairo_vm::Felt252;
use clap::Parser;
use prove_block::debug_prove_error;
use starknet_os::io::input::{Crdt, Slot};

const DEFAULT_COMPILED_OS: &[u8] = include_bytes!("../../../../build/os_latest.json");

#[derive(Parser, Debug)]
struct Args {
    /// Block to prove.
    #[arg(long = "block-number")]
    block_number: u64,

    /// RPC endpoint to use for fact fetching
    #[arg(long = "rpc-provider", default_value = "http://localhost:9545")]
    rpc_provider: String,

    #[arg(long = "shard_contract_address")]
    shard_contract_address: Felt252,
}

fn init_logging() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp(None)
        .try_init()
        .expect("Failed to configure env_logger");
}

#[tokio::main]
async fn main() {
    init_logging();

    let args = Args::parse();

    let block_number = args.block_number;
    let layout = LayoutName::all_cairo;
    let mut crdts = Vec::new();

    let crdt1 = Crdt {
        address: Felt252::from_hex_unchecked("0x325d09afe0dc15dc14967380e0017fad1f89e5ad8cc5663655f0637fdbbd01e"),
        slot_len: Felt252::from(2),
        slots: vec![
            Slot {
                key: Felt252::from_hex_unchecked("0x14de346fc4242ec4a43f4f2371dfae05680721698b5bba61a6f87c2fcf72de8"),
                crdt_type: Felt252::from(1),
            },
            Slot {
                key: Felt252::from_hex_unchecked("0x732ee9e7854af00ca86db8a297bb9f5e4da117ea0d934e18acd977d3b0a2a27"),
                crdt_type: Felt252::from(1),
            },
        ],
    };

    crdts.push(crdt1);
    let result =
        prove_block::prove_block(DEFAULT_COMPILED_OS, block_number, &args.rpc_provider, layout, true, crdts, true)
            .await;
    let (pie, _snos_output) = result.map_err(debug_prove_error).expect("Block proven");
    pie.run_validity_checks().expect("Valid PIE");
}
