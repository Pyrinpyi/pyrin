use crate::Hash;

#[derive(Clone)]
pub struct PowHash(blake3::Hasher);

#[derive(Clone)]
pub struct KHeavyHash;

impl PowHash {
    #[inline]
    pub fn new(pre_pow_hash: Hash, timestamp: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&pre_pow_hash.0);
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(&[0u8; 32]);
        Self (hasher)
    }

    #[inline(always)]
    pub fn finalize_with_nonce(&mut self, nonce: u64) -> Hash {
        self.0.update(&nonce.to_le_bytes());
        let mut hash_bytes = [0u8; 32];
        self.0.finalize_xof().fill(&mut hash_bytes);
        Hash::from_bytes(hash_bytes)
    }
}

impl KHeavyHash {
    #[inline]
    pub fn hash(in_hash: Hash) -> Hash {

        let bytes: &[u8] = &in_hash.0;
        let mut hasher = blake3::Hasher::new();
        hasher.update(bytes);

        let mut hash = [0u8; 32];
        hasher.finalize_xof().fill(&mut hash);
        Hash(hash)
    }
}

#[cfg(test)]
mod tests {
    use crate::Hash;

    use super::PowHash;

    #[test]
    fn test_pow_hash() {
        let timestamp: u64 = 1715521488610;
        let nonce: u64 = 11171827086635415026;
        let pre_pow_hash = Hash::from_bytes([
            99, 231, 29, 85, 153, 225, 235, 207, 36, 237, 3, 55, 106, 21, 221, 122, 28, 51, 249, 76, 190, 128, 153, 244, 189, 104, 26, 178, 170, 4, 177, 103
        ]);
        let mut hasher = PowHash::new(pre_pow_hash, timestamp);
        let hash1 = hasher.finalize_with_nonce(nonce);


        let mut hasher = blake3::Hasher::new();
        hasher
            .update(&pre_pow_hash.0)
            .update(&timestamp.to_le_bytes())
            .update(&[0u8; 32])
            .update(&nonce.to_le_bytes());

        let mut hash2 = [0u8; 32];
        hasher.finalize_xof().fill(&mut hash2);
        assert_eq!(Hash(hash2), hash1);
    }

    #[test]
    fn test_heavy_hash() {
        let timestamp: u64 = 1715521488610;
        let nonce: u64 = 11171827086635415026;
        let pre_pow_hash = Hash([42; 32]);

        let mut hasher = blake3::Hasher::new();
        hasher
            .update(&pre_pow_hash.0)
            .update(&timestamp.to_le_bytes())
            .update(&[0u8; 32])
            .update(&nonce.to_le_bytes());

        let mut hash1 = [0u8; 32];
        hasher.finalize_xof().fill(&mut hash1);

        let hash2 = Hash::from_bytes([
            0xe3, 0x69, 0xd9, 0xa9, 0xab, 0x79, 0x36, 0x22, 0xe8, 0xfc, 0x83, 0xb6, 0xbc, 0xb5,
            0x40, 0xf5, 0x53, 0x51, 0x0c, 0x8a, 0xdf, 0x5b, 0x9b, 0x99, 0x64, 0x98, 0x2e, 0x83,
            0x48, 0x11, 0x76, 0x26
        ]);
        assert_eq!(hash2, Hash(hash1));
    }
}
