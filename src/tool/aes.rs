use anyhow::{anyhow, Result};

// aes 128 解密
pub fn aes_16_ecb(plaintext: &str, key: &[u8; 16]) -> Result<String> {
    use data_encoding::BASE64;
    use openssl::symm::{decrypt, encrypt, Cipher, Crypter, Mode};

    let plaintext_data = BASE64
        .decode(plaintext.as_bytes())
        .map_err(|err| anyhow!("加密文件解码失败：{}", err))?;
    let cipher = Cipher::aes_128_ecb();

    let d =
        decrypt(cipher, key, None, &plaintext_data).map_err(|err| anyhow!("解密失败：{}", err))?;

    Ok(String::from_utf8(d)?)
}

// aes 128 解密
pub fn aes_24_ecb(plaintext: &str, key: &[u8; 24]) -> Result<String> {
    use data_encoding::BASE64;
    use openssl::symm::{decrypt, encrypt, Cipher, Crypter, Mode};

    let plaintext_data = BASE64
        .decode(plaintext.as_bytes())
        .map_err(|err| anyhow!("加密文件解码失败：{}", err))?;
    let cipher = Cipher::aes_192_ecb();

    let d =
        decrypt(cipher, key, None, &plaintext_data).map_err(|err| anyhow!("解密失败：{}", err))?;

    Ok(String::from_utf8(d)?)
}

pub fn aes_32_ecb(plaintext: &str, key: &[u8; 32]) -> Result<String> {
    use data_encoding::BASE64;
    use openssl::symm::{decrypt, encrypt, Cipher, Crypter, Mode};

    let plaintext_data = BASE64
        .decode(plaintext.as_bytes())
        .map_err(|err| anyhow!("加密文件解码失败：{}", err))?;
    let cipher = Cipher::aes_256_ecb();

    let d =
        decrypt(cipher, key, None, &plaintext_data).map_err(|err| anyhow!("解密失败：{}", err))?;

    Ok(String::from_utf8(d)?)
}
