use chacha20poly1305::{
    aead::{Aead, Payload},
    ChaCha20Poly1305, Key, KeyInit, Nonce,
};

pub(super) fn make_nonce(cid: &[u8; 8], seq: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(cid);
    nonce[8..12].copy_from_slice(&(seq as u32).to_le_bytes());
    nonce
}

pub(super) fn encrypt_with_key(
    key: &[u8; 32],
    cid: &[u8; 8],
    seq: u64,
    plaintext: &[u8],
) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce_bytes = make_nonce(cid, seq);
    let nonce = Nonce::from_slice(&nonce_bytes);
    cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad: &[],
            },
        )
        .expect("ChaCha20-Poly1305 encryption is infallible")
}

pub(super) fn decrypt_with_key(
    key: &[u8; 32],
    cid: &[u8; 8],
    seq: u64,
    ciphertext: &[u8],
) -> anyhow::Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce_bytes = make_nonce(cid, seq);
    let nonce = Nonce::from_slice(&nonce_bytes);
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: ciphertext,
                aad: &[],
            },
        )
        .map_err(|e| anyhow::anyhow!("decryption failed: {e:?}"))
}
