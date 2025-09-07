use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn main() {
    let seed = getrandom::u64().unwrap();
    let rng = ChaCha8Rng::seed_from_u64(seed);

    let tokio_seed = tokio::runtime::RngSeed::from_bytes(&seed.to_be_bytes());
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .rng_seed(tokio_seed)
        .build_local(Default::default())
        .unwrap();
}
