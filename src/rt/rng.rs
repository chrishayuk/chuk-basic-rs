//! Random numbers for RND and RANDOMIZE.

/// xorshift64* - small, fast and good enough for dice and card games.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // splitmix64 scramble so that small seeds still give a good state
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Rng((z ^ (z >> 31)) | 1)
    }

    /// Uniform in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let bits = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11;
        bits as f64 / (1u64 << 53) as f64
    }
}

/// A seed taken from the clock.
pub fn time_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x5EED)
}

/// The state behind `RND` and `RANDOMIZE`.
pub struct Rnd {
    rng: Rng,
    last: f64,
}

impl Rnd {
    /// `seed` fixes the sequence; `None` seeds from the clock.
    pub fn new(seed: Option<u64>) -> Self {
        Rnd {
            rng: Rng::new(seed.unwrap_or_else(time_seed)),
            last: 0.0,
        }
    }

    /// `RND(x)`: a positive `x` gives the next number in [0, 1), zero
    /// repeats the last one, and a negative `x` reseeds first.
    pub fn rnd(&mut self, x: f64) -> f64 {
        if x < 0.0 {
            self.rng = Rng::new(x.to_bits());
        }
        if x != 0.0 {
            self.last = self.rng.next_f64();
        }
        self.last
    }

    /// `RANDOMIZE [seed]`
    pub fn randomize(&mut self, seed: Option<f64>) {
        self.rng = Rng::new(seed.map_or_else(time_seed, f64::to_bits));
    }
}
