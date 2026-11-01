//! Aléa à compteur sans état : chaque tirage est une fonction pure de (graine, tick, cellule, flux).

const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Hachage SplitMix64 chaîné sur les quatre entrées.
pub fn hash(seed: u64, tick: u64, cell: u32, stream: u32) -> u64 {
    let mut state = mix(seed.wrapping_add(GAMMA));
    for word in [tick, u64::from(cell), u64::from(stream)] {
        state = mix(state.wrapping_add(GAMMA) ^ word);
    }
    state
}

/// Tirage uniforme dans [0, 1) à partir de 53 bits du hachage.
pub fn unit_f64(seed: u64, tick: u64, cell: u32, stream: u32) -> f64 {
    let bits = hash(seed, tick, cell, stream) >> 11;
    bits as f64 * (1.0 / 9_007_199_254_740_992.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_deterministic() {
        assert_eq!(hash(1, 2, 3, 4), hash(1, 2, 3, 4));
        assert_eq!(unit_f64(7, 8, 9, 10), unit_f64(7, 8, 9, 10));
        assert_ne!(hash(1, 2, 3, 4), hash(9, 2, 3, 4));
        assert_ne!(hash(1, 2, 3, 4), hash(1, 9, 3, 4));
        assert_ne!(hash(1, 2, 3, 4), hash(1, 2, 9, 4));
        assert_ne!(hash(1, 2, 3, 4), hash(1, 2, 3, 9));
    }

    #[test]
    fn rng_uniform_mean_and_bins() {
        const N: u32 = 1_000_000;
        let mut sum = 0.0;
        let mut bins = [0_u32; 10];
        for cell in 0..N {
            let u = unit_f64(42, 0, cell, 0);
            assert!((0.0..1.0).contains(&u));
            sum += u;
            bins[(u * 10.0) as usize] += 1;
        }
        assert!((sum / f64::from(N) - 0.5).abs() < 0.01);
        for count in bins {
            assert!((f64::from(count) / 1e5 - 1.0).abs() < 0.01, "bin {count}");
        }
    }

    #[test]
    fn rng_streams_independent() {
        const N: u32 = 1_000_000;
        let (mut sa, mut sb, mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for cell in 0..N {
            let a = unit_f64(42, 3, cell, 0) - 0.5;
            let b = unit_f64(42, 3, cell, 1) - 0.5;
            sa += a;
            sb += b;
            sab += a * b;
            saa += a * a;
            sbb += b * b;
        }
        let rho = sab / (saa * sbb).sqrt();
        assert!(rho.abs() < 0.005, "corrélation entre flux : {rho}");
        assert!((sa / f64::from(N)).abs() < 0.01 && (sb / f64::from(N)).abs() < 0.01);
    }
}
