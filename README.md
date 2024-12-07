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

## Benchmark results on the reference hardware

2024-12-07 10:21:22 Essential task `babe-worker` failed. Shutting down service.
2024-12-07 10:21:53 Block 25876038 with  4921 tx used 152.00% of its weight ( 3,039,966,059 of  1,999,979,786 ns) - OVER WEIGHT!
2024-12-07 10:22:21 Block 25876039 with  4561 tx used 141.43% of its weight ( 2,828,530,900 of  2,000,021,479 ns) - OVER WEIGHT!
2024-12-07 10:22:52 Block 25876040 with  5072 tx used 151.91% of its weight ( 3,037,951,145 of  1,999,873,477 ns) - OVER WEIGHT!
2024-12-07 10:23:06 Block 25876041 with  2318 tx used 205.63% of its weight ( 1,422,605,896 of    691,843,940 ns) - OVER WEIGHT!
2024-12-07 10:23:32 Block 25876042 with  4450 tx used 130.29% of its weight ( 2,605,683,476 of  1,999,903,633 ns) - OVER WEIGHT!
2024-12-07 10:23:59 Block 25876043 with  4785 tx used 137.75% of its weight ( 2,754,867,155 of  1,999,876,726 ns) - OVER WEIGHT!
2024-12-07 10:24:20 Block 25876044 with  3592 tx used 139.73% of its weight ( 2,029,712,277 of  1,452,554,205 ns) - OVER WEIGHT!
2024-12-07 10:24:45 Block 25876045 with  4870 tx used 131.49% of its weight ( 2,485,339,996 of  1,890,098,920 ns) - OVER WEIGHT!
2024-12-07 10:25:07 Block 25876046 with  4595 tx used 113.67% of its weight ( 2,273,344,776 of  1,999,945,390 ns) - OVER WEIGHT!
2024-12-07 10:25:30 Block 25876047 with  4552 tx used 115.02% of its weight ( 2,300,202,614 of  1,999,877,316 ns) - OVER WEIGHT!
2024-12-07 10:25:54 Block 25876048 with  4914 tx used 117.71% of its weight ( 2,354,063,571 of  1,999,918,739 ns) - OVER WEIGHT!


