use crate::error::Result;
use crate::fs::File;
use crate::io::Read;
use rusl::error::Errno;

/// Fills the provided buffer with random bytes from /dev/random.
/// # Errors
/// File permissions failure
pub fn system_random(buf: &mut [u8]) -> Result<()> {
    let mut file = File::open("/dev/random\0")?;
    let mut offset = 0;
    while offset < buf.len() {
        match file.read(&mut buf[offset..]) {
            Ok(read) => {
                offset += read;
            }
            Err(e) => {
                if e.matches_errno(Errno::EINTR) {
                    continue;
                }
                return Err(e);
            }
        }
    }
    Ok(())
}

/// A small, basic [PRNG](https://en.wikipedia.org/wiki/Pseudorandom_number_generator)
/// implemented as an [LCG](https://en.wikipedia.org/wiki/Linear_congruential_generator).
/// Should never be used for security-critical things, but can be used as a high-performance
/// source of pseudo-randomness.
/// For secure randoms [`system_random`] is more appropriate.
pub struct Prng {
    seed: u64,
}

impl Prng {
    /// Choosing same constants as glibc here.
    /// [LCG](https://en.wikipedia.org/wiki/Linear_congruential_generator)
    /// Yanked my implementation from [tiny-bench, license here](https://github.com/EmbarkStudios/tiny-bench/blob/main/LICENSE-MIT)
    const MOD: u128 = 2u128.pow(48);
    const A: u128 = 25_214_903_917;
    const C: u128 = 11;

    /// Create a new Prng-instance seeding with a `MonotonicInstant`, this is fine because
    /// you shouldn't need a secure seed anyway, because you should not use this for
    /// security-purposes at all.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn new_time_seeded() -> Self {
        let time = crate::time::MonotonicInstant::now();
        let time_nanos_in_u64 = (time.0.seconds() as u64)
            .overflowing_add(time.0.nanoseconds() as u64)
            .0;
        Prng {
            seed: time_nanos_in_u64,
        }
    }

    /// Create a new prng-instance with the provided seed
    #[inline]
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Prng {
            // And maybe check for overflows. Note: No we're good until about year 2554
            seed,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.seed = ((Self::A * u128::from(self.seed) + Self::C) % Self::MOD) as u64;
        self.seed
    }
}

impl Iterator for Prng {
    type Item = u64;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_random() {
        let mut buf = [0u8; 4096];
        system_random(&mut buf).unwrap();
        // Likely outcome is 16 around zeroes
        let mut count_zero = 0;
        for i in buf {
            if i == 0 {
                count_zero += 1;
            }
        }
        // Could calculate the actual probability for this.
        assert!(count_zero < 32, "After filling a buf with random bytes {count_zero} zeroes were found, should be around 16.");
    }

    #[test]
    fn gets_pseudo_random() {
        let mut count_zero = 0;
        let prng = Prng::new(55);
        for val in prng.take(4096) {
            if val == 0 {
                count_zero += val;
            }
        }
        // Sweet determinism
        assert_eq!(0, count_zero);
    }
}
