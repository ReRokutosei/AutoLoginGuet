// 密码加密解密模块，使用AES-CBC加密算法对密码进行加密存储

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use winreg::enums::*;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

/// 从密码派生密钥
fn derive_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// 加密密码
pub fn encrypt_password(password: &str, key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let key = derive_key(key);
    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);
    
    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let mut buffer = vec![0u8; password.len() + 16];
    buffer[..password.len()].copy_from_slice(password.as_bytes());
    
    let ciphertext = cipher.encrypt_padded_mut::<Pkcs7>(&mut buffer, password.len())
        .map_err(|e| format!("加密失败: {:?}", e))?;
    
    let mut result = Vec::with_capacity(iv.len() + ciphertext.len());
    result.extend_from_slice(&iv);
    result.extend_from_slice(ciphertext);
    Ok(general_purpose::STANDARD.encode(result))
}

/// 解密密码
pub fn decrypt_password(encrypted_password: &str, key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let key = derive_key(key);
    let data = general_purpose::STANDARD.decode(encrypted_password)
        .map_err(|e| format!("Base64解码失败: {:?}", e))?;
    
    if data.len() < 16 {
        return Err("数据长度不足".into());
    }
    
    let (iv, ciphertext) = data.split_at(16);
    
    let cipher = Aes256CbcDec::new(&key.into(), iv.into());
    let mut buffer = vec![0u8; ciphertext.len()];
    buffer[..ciphertext.len()].copy_from_slice(ciphertext);
    
    let plaintext = cipher.decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|e| format!("解密失败: {:?}", e))?;
    
    Ok(String::from_utf8(plaintext.to_vec())
        .map_err(|e| format!("UTF-8解码失败: {:?}", e))?)
}

#[cfg(windows)]
/// 生成机器相关的密钥（基于机器信息）
pub fn generate_machine_key() -> String {
    let machine_key = get_machine_guid().unwrap_or_else(|_| {
        "AutoLoginGUET_default_key_2025".to_string()
    });
    
    format!("AutoLoginGUET_salt_2025_{}", machine_key)
}

#[cfg(windows)]
/// 获取Windows机器GUID
fn get_machine_guid() -> Result<String, Box<dyn std::error::Error>> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey("SOFTWARE\\Microsoft\\Cryptography")?;
    let machine_guid: String = key.get_value("MachineGuid")?;
    Ok(machine_guid)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let password = "test_password_123";
        let key = "my_secret_key";
        
        let encrypted = encrypt_password(password, key).unwrap();
        let decrypted = decrypt_password(&encrypted, key).unwrap();
        
        assert_eq!(password, decrypted);
    }
}