// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, BatchSize};
use fractal_utils::channel;
use winter_crypto::{build_merkle_nodes, hashers::Blake3_256, Hasher, MerkleTree, RandomCoin};
use winter_math::fields::f128::BaseElement;
use winter_rand_utils::rand_value;
use winter_utils::uninit_vector;

type Blake3 = Blake3_256<BaseElement>;
type Blake3Digest = <Blake3 as Hasher>::Digest;

pub fn merkle_tree_construction(c: &mut Criterion) {
    let mut merkle_group = c.benchmark_group("merkle tree batch proving");
    merkle_group.sample_size(10);
    // static BATCH_SIZES: [usize; 3] = [65536, 131072, 262144];
    // static BATCH_SIZES: [usize; 2] = [16 * 64, 2048 * 4 * 2];
    let leaf_sizes = (1..25);

    

    for size_log in leaf_sizes {
        let size = 1<<size_log;
        let proof_sizes = (1..size_log - 1).map(|x| 1 << x);
        let data: Vec<Blake3Digest> = {
            let mut res = unsafe { uninit_vector(size) };
            for i in 0..size {
                res[i] = Blake3::hash(&rand_value::<u128>().to_le_bytes());
            }
            res
        };

        let com_tree = MerkleTree::<Blake3>::new(data).unwrap();
        let seed = [0u8];
        let mut coin = RandomCoin::<BaseElement, Blake3>::new(&seed);
        
        for query_count in proof_sizes {
            let queries = coin.draw_integers(query_count, size).unwrap();
            merkle_group.bench_function(BenchmarkId::new("Proving batch for", format!("Tree of size {:?}, queries {:?}", size, query_count)), |bench| {
                bench.iter_batched_ref(
                    || queries.clone(),
                    |p| com_tree.prove_batch(&queries),
                    BatchSize::LargeInput,
                );
            });
        }
        
        // merkle_group.bench_with_input(BenchmarkId::new("concurrent", size), &data, |b, i| {
        //     b.iter(|| concurrent::build_merkle_nodes::<Blake3>(&i))
        // });
    }
}

criterion_group!(merkle_group, merkle_tree_construction,);
criterion_main!(merkle_group);
