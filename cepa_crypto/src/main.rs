use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use rand::RngCore;
use rsa::{Oaep, PublicKey, RsaPrivateKey, RsaPublicKey};

fn encrypt_rsa(pub_key: &RsaPublicKey, msg: &[u8]) -> Vec<u8> {
    let mut rng = rand::thread_rng();

    let padding = Oaep::new::<sha2::Sha256>();
    pub_key
        .encrypt(&mut rng, padding, msg)
        .expect("failed to encrypt RSA")
}

fn decrypt_rsa(priv_key: &RsaPrivateKey, enc_msg: &[u8]) -> Vec<u8> {
    let padding = Oaep::new::<sha2::Sha256>();
    priv_key
        .decrypt(padding, &enc_msg)
        .expect("failed to decrypt RSA")
}

fn encrypt_aes(msg: &[u8]) -> ([u8; 32], [u8; 12], Vec<u8>) {
    let mut rng = rand::thread_rng();

    let mut key_bytes = [0u8; 32];
    rng.fill_bytes(&mut key_bytes);

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

    let mut nonce_bytes = [0u8; 12];
    rng.fill_bytes(&mut nonce_bytes);

    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new(&key);
    let enc_msg = cipher.encrypt(nonce, msg).expect("failed to encrypt AES");

    (key_bytes, nonce_bytes, enc_msg)
}

fn decrypt_aes(key_bytes: &[u8; 32], nonce_bytes: &[u8; 12], enc_msg: &[u8]) -> Vec<u8> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);

    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(&key);

    cipher
        .decrypt(nonce, enc_msg)
        .expect("failed to decrypt AES")
}

fn main() {
    let bits = 2048;

    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    let message = b"Hello Cepa";

    // Encrypt message with AES
    let (aes_key, aes_nonce, rsa_enc_msg) = encrypt_aes(message);

    // Put AES key and nonce in the same array
    let mut aes_key_nonce = [0u8; 44];
    aes_key_nonce[0..32].copy_from_slice(&aes_key);
    aes_key_nonce[32..44].copy_from_slice(&aes_nonce);

    // Encrypt AES key and nonce with rsa
    let enc_key_nonce = encrypt_rsa(&public_key, &aes_key_nonce);

    // Decrypt AES key and nonce with rsa
    let dec_key_nonce = decrypt_rsa(&private_key, &enc_key_nonce);

    // Retreive key and nonce from the decrypted array
    let mut dec_aes_key = [0u8; 32];
    let mut dec_aes_nonce = [0u8; 12];

    dec_aes_key.copy_from_slice(&dec_key_nonce[0..32]);
    dec_aes_nonce.copy_from_slice(&dec_key_nonce[32..44]);

    let dec_message = decrypt_aes(&dec_aes_key, &dec_aes_nonce, &rsa_enc_msg);

    println!("{}", String::from_utf8_lossy(&dec_message));
}
