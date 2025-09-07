use crate::env::Env;

mod cluster;
mod env;
mod log;

fn main() {
    let seed = getrandom::u64().unwrap();

    let env = Env::new(seed);

    let tokio_seed = tokio::runtime::RngSeed::from_bytes(&seed.to_be_bytes());
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .rng_seed(tokio_seed)
        .build_local(Default::default())
        .unwrap();
}
