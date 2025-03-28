use std::{path::{Path, PathBuf}, str::FromStr};
use starknet_types_core::felt::Felt;
use cairo_vm::types::layout_name::LayoutName;
use clap::Parser;
use prove_block::debug_prove_error;

const DEFAULT_COMPILED_OS: &[u8] = include_bytes!("../../../../build/snos.json");

#[derive(Parser, Debug)]
struct Args {
    /// Block to prove.
    #[arg(long = "block-number")]
    block_number: u64,

    /// RPC endpoint to use for fact fetching
    #[arg(long = "rpc-provider", default_value = "http://localhost:9545")]
    rpc_provider: String,
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

    let result = prove_block::prove_block(DEFAULT_COMPILED_OS, block_number, &args.rpc_provider, layout, true).await;
    let (pie, _snos_output) = result.map_err(debug_prove_error).expect("Block proven");
    pie.write_zip_file(&PathBuf::from_str("block.zip").unwrap()).expect("Zip file written");
    
    let output_segment_index = 2_usize;
    let output_segment = prove_block::get_memory_segment(&pie, output_segment_index);
    let output: Vec<Felt> = output_segment
        .iter()
        .map(|(_key, value)| value.get_int().unwrap())
        .collect::<Vec<_>>();

    let output_json = serde_json::to_string(&output)
        .expect("Failed to serialize output to JSON");
    std::fs::write(Path::new(format!("snos_output_{}.json", block_number).as_str()), output_json).expect("Failed to write json file");
    let serialized_os_output = serde_json::to_string(&_snos_output)
        .expect("Failed to serialize output to JSON");
    std::fs::write(Path::new(format!("os_output_{}.json", block_number).as_str()), serialized_os_output).expect("Failed to write json file");
    pie.run_validity_checks().expect("Valid PIE");
}
