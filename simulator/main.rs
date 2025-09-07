use bytes::Bytes;
use rand::{Rng, RngCore};

use crate::{cluster::Cluster, env::Env};

mod clock;
mod cluster;
mod env;
mod log;
mod oracle;

fn main() {
    let seed = getrandom::u64().unwrap();

    let mut env = Env::new(seed);

    let tokio_seed = tokio::runtime::RngSeed::from_bytes(&seed.to_be_bytes());
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .rng_seed(tokio_seed)
        .build_local(Default::default())
        .unwrap();

    // TODO: vary node counter
    let cluster = Cluster::new(3, &mut env.rng);

    let num_proposals = env.rng.random_range(10..20);
    for _ in 0..num_proposals {
        let mut data: [u8; 128] = [0; 128];
        env.rng.fill_bytes(&mut data);
        let data = Bytes::from_owner(data);
    }
}
