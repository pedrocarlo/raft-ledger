use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub struct Env {
    pub seed: u64,
    pub rng: ChaCha8Rng,
}

impl Env {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }
}
