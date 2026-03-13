pub fn splitmix_u64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

pub fn splitmix_f32(seed: u64) -> f32 {
    (splitmix_u64(seed) as f64 / u64::MAX as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::splitmix_u64;

    #[test]
    fn splitmix_u64_known_vectors_are_stable() {
        assert_eq!(splitmix_u64(0), 0xE220_A839_7B1D_CDAF);
        assert_eq!(splitmix_u64(1), 0x910A_2DEC_8902_5CC1);
        assert_eq!(splitmix_u64(42), 0xBDD7_3226_2FEB_6E95);
        assert_eq!(splitmix_u64(0xDEAD_BEEF), 0x4ADF_B90F_68C9_EB9B);
    }
}
