# Disruptor-rs

High performance application framework based on [LMAX](https://lmax-exchange.github.io/disruptor/).

## WARNING
This is pre-alpha stuff, no need to even look at this yet.

## TODO

[ ] Benchmark the throughput.

[ ] Add a scheduler to the reactor which, based on a configured number of threads, only lets a single processing core
`sink` the result, while the others already precompute the next result.

[ ] Implement LMAX's lockless broker.

[ ] Correctly handle errors. (Many unwraps currently remain.)

[ ] Write documentation and examples.