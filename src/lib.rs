#![cfg(test)]

use frame_remote_externalities::{Builder, Mode, OfflineConfig, SnapshotConfig};
use runtime::*;
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

    let _ = Executive::finalize_block();
    eprintln!(
        "Time to import #{}: {}ms",
        System::block_number(),
        timer.elapsed().as_millis()
    );
}

#[tokio::test]
async fn main() {
    sp_tracing::try_init_simple();

    let blocks: Vec<Block> = [
        "0xfd120b1ebf45b363b4bd4fa212f8f79acc52797dfa04fff873009ea6314bc8be",
        "0xcae5f6268b12f6203ed3734b448d67631f987686a78ba97e1b57b52149b9f413",
        "0x52875b8a8e68f607c166c106fd19b78236b6362ec6d4d44104939ba6bce9c97b",
        "0x211ead00957c08922b9f4502da4d991bccef5bde581826c45344d2794ed3d312",
        "0x949f72003b60b36c9fb65fedcabfd145af90141b5bb02a1664d004756c7b9e07",
        "0xaaa66bfa64e4111feba52a66b9b14ee4ee199f1c5ccdcc24d361da02bf67189a",
        "0x2a2fed8672fcdb999d289ba101be42e4640a9ae6fd5d5c06ebec3416a6be9df3",
        "0xfbe5616dcb5490dddd0de68c95fd41c8fadb972dc0d0ee4524bb6fec962b0a04",
        "0xbef56aa82850c3515de4ed18487320399f221275ff7cf2625f8c67cf3b1925b1",
        "0xf1707fdd54d9d4feea6c4a326e4f601c817c38849f6b844f32b5f372333ad563",
        "0xa13dad52e98860b5c50905046cf43165650aff2e4be56cbf7b92e5eb6f26a6de",
    ]
    .into_iter()
    .map(|hash| {
        let file = File::open(format!("lfs/block-{hash}.raw")).expect("Block file not found");
        let reader = BufReader::new(file);
        let block: Block = serde_json::from_reader(reader).expect("Block decoding failed");
        block
    })
    .collect();

    let state_snapshot = SnapshotConfig::new(
        "lfs/snap-0xb131e2457a0bd3ba3395319c84715cc136354cb31e1901e731a1de1c82fc3a68.raw",
    );
    Builder::<Block>::default()
        .mode(Mode::Offline(OfflineConfig { state_snapshot }))
        .build()
        .await
        .unwrap()
        .execute_with(|| {
            for block in blocks {
                replay(block);
            }
        });
}
