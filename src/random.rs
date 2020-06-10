use rand::prelude::*;
use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    os::raw::c_int,
};

#[derive(PartialEq, Eq, Hash)]
pub struct Seed {
    pub x: c_int,
    pub y: c_int,
    pub z: c_int,
    pub volume: c_int,
}
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn get_rng(seed: &Seed) -> Box<dyn RngCore + Send> {
    let hash = calculate_hash(seed);

    Box::new(ChaChaRng::seed_from_u64(hash))
}
