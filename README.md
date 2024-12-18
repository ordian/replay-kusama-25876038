# Debugging Kusama block import time

## Incident

On Kusama during relay chain spamming event from Amforc, block import times spiked above 2s. That could indicate potentially
underestimate in weights. The block 25876038 was filled with 4919 `transfer_keep_alive` transactions:

```
# Running `polkadot benchmark block --from 25876038 --to 25876039`
2024-11-28 07:00:39 Block 25876038 with  4921 tx used 122.78% of its weight ( 2,455,672,713 of  1,999,979,786 ns) - OVER WEIGHT!
```
This was run on an archive node: https://github.com/paritytech/devops/issues/3616#issuecomment-2505398706.

with `paraInherent.enter` taking reportedly 27.58% of the 2s weight (551,758,840,300 ns). At the time of the event, the weight of
`paraInherent.enter` could be an [underestimate](https://github.com/paritytech/polkadot-sdk/issues/849#issuecomment-2345949574) since
it does not include the weights of enacting candidates and even though the fixes were merged into polkadot-sdk, they haven't been
enacted on kusama yet.

## Replay

Equipped with https://github.com/ggwpez/wtfwt/, we can obtain a snapshot of the previous block state and download the block in order to replay it.
This was done with a simple command:

```
# make sure to install `try-runtime-cli` beforehand: cargo install --git https://github.com/paritytech/try-runtime-cli --locked
/target/release/wtfwt --rpc wss://kusama-rpc.polkadot.io --block 0xfd120b1ebf45b363b4bd4fa212f8f79acc52797dfa04fff873009ea6314bc8be --runtime-name staging-kusama --source-repo "polkadot-fellows/runtimes" --source-rev "v1.3.4" --force
```

I've moved the state and block data into lfs folder since they don't fit into git file limits and used git-lfs to track them.

This should generate a `replay` folder with a test that imports that block. Note that import will be run in native (not wasm) execution mode, so it might have better performance than wasm. However, it makes debugging experience easier.

Running `cargo test --release -- --nocapture` show that import takes 750ms on M1 MAX laptop. If we assume 1.5x performance boost from using native and another 1.5x from the faster hardware compared to the reference one, we should still be within of allowed 2s execution time.

## Open questions

* why are the blocks actually take more than 2s on the archive node to execute? is it coming from wasm vs native, CPU/mem differences?
* I've noticed that after rerunning the execution locally, it takes around 550ms (vs 750ms). this suggests that the execution is IO bound and caching helps? or could it be CPU branch prediction?
* why did it take substantially longer of validators to import than 2s? CPU starvation due to PVF checks?

## Next steps

- try rerunning on reference hardware (the archive node seems underspecced and was syncing in parallel with the bench)
- try increasing the trie cache size (`--trie-cache-size 1073741824`) and see how the results are different
