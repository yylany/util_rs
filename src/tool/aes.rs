use aes::cipher::{generic_array::GenericArray, BlockDecrypt};
use aes::{Aes128, NewBlockCipher};
use anyhow::{anyhow, Result};

fn decrypt_aes_128_ecb(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 16 {
        return Err(anyhow!("密钥长度必须为 128 位（16 字节）"));
    }
    if ciphertext.len() % 16 != 0 {
        return Err(anyhow!("密文长度必须是 16 的倍数"));
    }

    let key = GenericArray::from_slice(key);
    let cipher = Aes128::new(key);

    let mut buffer = Vec::from(ciphertext);
    for chunk in buffer.chunks_mut(16) {
        cipher.decrypt_block(GenericArray::from_mut_slice(chunk));
    }

    Ok(buffer)
}

// aes 128 解密
pub fn aes_16_ecb(plaintext: &str, key: &[u8; 16]) -> Result<String> {
    use data_encoding::BASE64;

    let plaintext_data = BASE64
        .decode(plaintext.as_bytes())
        .map_err(|err| anyhow!("加密文件解码失败：{}", err))?;

    let d = String::from_utf8(decrypt_aes_128_ecb(&plaintext_data, key)?)?
        .trim()
        .to_string();

    Ok(d)
}
