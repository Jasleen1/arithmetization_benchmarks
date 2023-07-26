// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use winter_crypto::{build_merkle_nodes, hashers::Blake3_256, Hasher};
use winter_math::fields::f128::BaseElement;
use winter_rand_utils::rand_value;
use winter_utils::uninit_vector;

type Blake3 = Blake3_256<BaseElement>;
type Blake3Digest = <Blake3 as Hasher>::Digest;

pub fn merkle_tree_construction(c: &mut Criterion) {
    let mut merkle_group = c.benchmark_group("merkle tree construction");

    // static BATCH_SIZES: [usize; 3] = [65536, 131072, 262144];
    static BATCH_SIZES: [usize; 2] = [16 * 64, 2048 * 4 * 2];

    for size in &BATCH_SIZES {
        let data: Vec<Blake3Digest> = {
            let mut res = unsafe { uninit_vector(*size) };
            for i in 0..*size {
                res[i] = Blake3::hash(&rand_value::<u128>().to_le_bytes());
            }
            res
        };
        merkle_group.bench_with_input(BenchmarkId::new("sequential", size), &data, |b, i| {
            b.iter(|| build_merkle_nodes::<Blake3>(&i))
        });
        // merkle_group.bench_with_input(BenchmarkId::new("concurrent", size), &data, |b, i| {
        //     b.iter(|| concurrent::build_merkle_nodes::<Blake3>(&i))
        // });
    }
}

criterion_group!(merkle_group, merkle_tree_construction,);
criterion_main!(merkle_group);
