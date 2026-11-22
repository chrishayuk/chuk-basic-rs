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
    /// Whether the run was given a seed, so that it must repeat exactly.
    seeded: bool,
}

impl Rnd {
    /// `seed` fixes the sequence; `None` seeds from the clock.
    pub fn new(seed: Option<u64>) -> Self {
        Rnd {
            rng: Rng::new(seed.unwrap_or_else(time_seed)),
            last: 0.0,
            seeded: seed.is_some(),
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

    /// `RANDOMIZE [seed]`. Without a seed it reseeds from the clock - unless
    /// the run itself was seeded (`--seed`), when it reseeds from the
    /// generator, so that a seeded run repeats exactly. The playground
    /// relies on that: it replays a session from the start at every `INPUT`.
    pub fn randomize(&mut self, seed: Option<f64>) {
        let seed = match seed {
            Some(seed) => seed.to_bits(),
            None if self.seeded => self.rng.next_f64().to_bits(),
            None => time_seed(),
        };
        self.rng = Rng::new(seed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_seeded_run_repeats_even_after_a_bare_randomize() {
        let run = || {
            let mut rnd = Rnd::new(Some(42));
            let first = rnd.rnd(1.0);
            rnd.randomize(None);
            (first, rnd.rnd(1.0), rnd.rnd(1.0))
        };
        assert_eq!(run(), run());
        // and RANDOMIZE still changes the sequence
        let mut plain = Rnd::new(Some(42));
        plain.rnd(1.0);
        assert_ne!(plain.rnd(1.0), run().1);
    }
}
