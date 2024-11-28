#![cfg(test)]

use runtime::*;
use frame_remote_externalities::{
	Builder, Mode, OfflineConfig, SnapshotConfig,
};
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

// Your replay logic here:
//
// This can be treated as a unit test like in the Polkadot-SDK repo. The runtime is in scope as
// `runtime` and pallets can be accessed like eg `System::block_number()`.
fn replay(block: Block) {
    let timer = Instant::now();
	Executive::initialize_block(&block.header);

	for extrinsic in block.extrinsics {
		let _ = Executive::apply_extrinsic(extrinsic);
	}

	//eprintln!("Events: {:#?}", System::events());

	let _ = Executive::finalize_block();
	eprintln!("Time to import: {}ms", timer.elapsed().as_millis());
}

#[tokio::test]
async fn main() {
	sp_tracing::try_init_simple();

	let raw_block = File::open("lfs/block-0xfd120b1ebf45b363b4bd4fa212f8f79acc52797dfa04fff873009ea6314bc8be.raw")
		.expect("Block file not found");
    let reader = BufReader::new(raw_block);
	let block: Block = serde_json::from_reader(reader)
		.expect("Block decoding failed");
	
	let state_snapshot = SnapshotConfig::new("lfs/snap-0xb131e2457a0bd3ba3395319c84715cc136354cb31e1901e731a1de1c82fc3a68.raw");
	Builder::<Block>::default()
		.mode(Mode::Offline(
			OfflineConfig { state_snapshot },
		))
		.build()
		.await
		.unwrap()
		.execute_with(|| {
			replay(block);
		});
}
