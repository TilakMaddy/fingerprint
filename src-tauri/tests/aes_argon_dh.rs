#[cfg(test)]
mod encryption_and_hashing_library_tests {

    use aes_gcm::{
        aead::{Aead, AeadCore, KeyInit, OsRng},
        Aes256Gcm, Error as AesError, Key,
    };

    use argon2::{password_hash::SaltString, Argon2};

    use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

    #[test]
    fn test_basic_encryption_from_random_key() -> Result<(), AesError> {
        let key = Aes256Gcm::generate_key(OsRng);

        let cipher = Aes256Gcm::new(&key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message

        let ciphertext = cipher.encrypt(&nonce, b"plaintext message".as_ref())?;
        let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref())?;
        assert_eq!(&plaintext, b"plaintext message");

        Ok(())
    }

    #[test]
    fn test_encryption_from_custom_string_key() -> Result<(), AesError> {
        // Custom string key
        let user_password = String::from("milton_bottles_k3y");

        let salt = SaltString::generate(&mut OsRng);
        let mut salt_bytes = [0u8; 16];
        let salt_bytes = salt
            .decode_b64(&mut salt_bytes)
            .expect("decoding doesnt work");

        let mut argon_output_key = [0u8; 32]; // 256 bit key
        Argon2::default()
            .hash_password_into(user_password.as_bytes(), salt_bytes, &mut argon_output_key)
            .expect("argon hashing failed");

        let aes_key: Key<Aes256Gcm> = argon_output_key.into();
        let cipher = Aes256Gcm::new(&aes_key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message

        let ciphertext = cipher.encrypt(&nonce, b"plaintext message".as_ref())?;
        let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref())?;
        assert_eq!(&plaintext, b"plaintext message");

        Ok(())
    }

    #[test]
    fn test_key_exchange_works() {
        let alice_secret = StaticSecret::random();
        let alice_public = PublicKey::from(&alice_secret);
        let alice_public2 = PublicKey::from(&alice_secret);

        assert!(alice_public.to_bytes() == alice_public2.to_bytes());

        let bob_secret = EphemeralSecret::random();
        let bob_public = PublicKey::from(&bob_secret);

        let asb = alice_secret.to_bytes();
        let asss = StaticSecret::from(asb);

        let alice_shared_secret = asss.diffie_hellman(&bob_public);
        let bob_shared_secret = bob_secret.diffie_hellman(&alice_public);

        assert_eq!(alice_shared_secret.as_bytes(), bob_shared_secret.as_bytes());
    }
}
