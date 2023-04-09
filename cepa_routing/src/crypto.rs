use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use rand::RngCore;
use rsa::{Oaep, PublicKey, RsaPrivateKey, RsaPublicKey};

pub fn encrypt_rsa(pub_key: &RsaPublicKey, msg: &[u8]) -> Vec<u8> {
    let mut rng = rand::thread_rng();

    let padding = Oaep::new::<sha2::Sha256>();
    pub_key
        .encrypt(&mut rng, padding, msg)
        .expect("failed to encrypt RSA")
}

pub fn decrypt_rsa(priv_key: &RsaPrivateKey, enc_msg: &[u8]) -> Vec<u8> {
    let padding = Oaep::new::<sha2::Sha256>();
    priv_key
        .decrypt(padding, enc_msg)
        .expect("failed to decrypt RSA")
}

pub fn encrypt_aes(msg: &[u8]) -> ([u8; 32], [u8; 12], Vec<u8>) {
    let mut rng = rand::thread_rng();

    let mut key_bytes = [0u8; 32];
    rng.fill_bytes(&mut key_bytes);

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

    let mut nonce_bytes = [0u8; 12];
    rng.fill_bytes(&mut nonce_bytes);

    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new(key);
    let enc_msg = cipher.encrypt(nonce, msg).expect("failed to encrypt AES");

    (key_bytes, nonce_bytes, enc_msg)
}

pub fn decrypt_aes(key_bytes: &[u8; 32], nonce_bytes: &[u8; 12], enc_msg: &[u8]) -> Vec<u8> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);

    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(key);

    cipher
        .decrypt(nonce, enc_msg)
        .expect("failed to decrypt AES")
}

pub fn wrap_layer(dest_pub_key: &RsaPublicKey, dest_ip: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let (aes_key, aes_nonce, mut aes_encrypted) = encrypt_aes(payload);

    let mut header = aes_key.to_vec();
    header.extend_from_slice(&aes_nonce);
    header.extend_from_slice(dest_ip);

    let mut rsa_encrypted = encrypt_rsa(dest_pub_key, &header);

    rsa_encrypted.append(&mut aes_encrypted);

    rsa_encrypted
}

pub fn unwrap_layer(priv_key: &RsaPrivateKey, encrypted_data: &[u8]) -> ([u8; 4], Vec<u8>) {
    let (rsa_encrypted, aes_encrypted) = encrypted_data.split_at(256);

    let rsa_decrypted = decrypt_rsa(priv_key, rsa_encrypted);

    let mut aes_key = [0u8; 32];
    let mut aes_nonce = [0u8; 12];
    let mut dest_ip = [0u8; 4];

    aes_key.copy_from_slice(&rsa_decrypted[0..32]);
    aes_nonce.copy_from_slice(&rsa_decrypted[32..44]);
    dest_ip.copy_from_slice(&rsa_decrypted[44..48]);

    (dest_ip, decrypt_aes(&aes_key, &aes_nonce, aes_encrypted))
}

#[test]
fn main() {
    let bits = 2048;

    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    let message = b"Hello Cepa";

    let wrapped = wrap_layer(&public_key, &[127, 0, 0, 0], message);

    let (dest_ip, unwrapped) = unwrap_layer(&private_key, &wrapped);

    assert_eq!(dest_ip, [127, 0, 0, 0]);
    assert_eq!(unwrapped, message);
}
