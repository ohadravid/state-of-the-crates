#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn hash_test() {
        let mut hashmap = HashMap::<&'static str, usize, rustc_hash::FxBuildHasher>::default();

        hashmap.insert("black", 0);
        hashmap.insert("white", 255);
    }

    #[test]
    fn sha1_test() {
        let mut m = sha1_smol::Sha1::new();
        m.update(b"Hello 2025!");
        let hash = m.digest().to_string();
        assert_eq!(hash, "68a0e44442b100a1698afa962aa590cc5e1cbb71");
    }
}
