//! 密码加密解密模块

use crate::core::error::{AppError, AppResult};
use crate::core::events::{EventBus, notify_login_attempted, notify_network_status_checked};
use crate::core::message::{CampusNetworkStatus, WanStatus};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;
use sha2::{Digest, Sha256};
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
    hasher.update(b"AutoLoginGUET_SALT_2025");
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// 加密密码
///
/// 使用AES-CBC算法加密密码，需要提供加密密钥
///
/// # 参数
/// * `password` - 需要加密的明文密码
/// * `key` - 用于加密的密钥
///
/// # 返回值
/// 返回加密后的密码字符串，或包含错误信息的AppError
pub fn encrypt_password(password: &str, key: &str) -> AppResult<String> {
    let key = derive_key(key);
    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);

    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let mut buffer = vec![0u8; password.len() + 16];
    buffer[..password.len()].copy_from_slice(password.as_bytes());

    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, password.len())
        .map_err(|e| AppError::CryptoError(format!("加密失败: {:?}", e)))?;

    let mut result = Vec::with_capacity(iv.len() + ciphertext.len());
    result.extend_from_slice(&iv);
    result.extend_from_slice(ciphertext);
    Ok(general_purpose::STANDARD.encode(&result))
}

/// 解密密码
///
/// 使用AES-CBC算法解密密码，需要提供解密密钥
///
/// # 参数
/// * `encrypted_password` - 需要解密的密文密码
/// * `key` - 用于解密的密钥
///
/// # 返回值
/// 返回解密后的明文密码，或包含错误信息的AppError
pub fn decrypt_password(encrypted_password: &str, key: &str) -> AppResult<String> {
    let key = derive_key(key);
    let data = general_purpose::STANDARD
        .decode(encrypted_password)
        .map_err(|e| AppError::CryptoError(format!("Base64解码失败: {:?}", e)))?;

    if data.len() < 16 {
        return Err(AppError::CryptoError("数据长度不足".into()));
    }

    let (iv, ciphertext) = data.split_at(16);

    let cipher = Aes256CbcDec::new(&key.into(), iv.into());
    let mut buffer = vec![0u8; ciphertext.len()];
    buffer[..ciphertext.len()].copy_from_slice(ciphertext);

    let plaintext = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|e| AppError::CryptoError(format!("解密失败: {:?}", e)))?;

    String::from_utf8(plaintext.to_vec())
        .map_err(|e| AppError::CryptoError(format!("UTF-8解码失败: {:?}", e)))
}

#[cfg(windows)]
/// 获取Windows机器GUID
fn get_machine_guid() -> AppResult<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey("SOFTWARE\\Microsoft\\Cryptography")
        .map_err(|e| AppError::CryptoError(format!("无法打开注册表项: {}", e)))?;
    let machine_guid: String = key
        .get_value("MachineGuid")
        .map_err(|e| AppError::CryptoError(format!("无法获取MachineGuid值: {}", e)))?;
    Ok(machine_guid)
}

#[cfg(windows)]
/// 生成机器相关的密钥（基于机器信息）
///
/// 用于生成与机器绑定的加密密钥
/// 如果无法获取机器信息，则使用默认密钥
pub fn generate_machine_key() -> String {
    match get_machine_guid() {
        Ok(guid) => {
            format!("AutoLoginGUET_salt_2025_{}", guid)
        }
        Err(_) => "AutoLoginGUET_default_key_2025".to_string(),
    }
}

/// 使用机器密钥加密密码
///
/// 结合了密钥生成和密码加密两个步骤的便捷函数
///
/// # 参数
/// * `password` - 需要加密的明文密码
///
/// # 返回值
/// 返回加密后的密码字符串，或包含错误信息的AppError
pub fn encrypt_password_with_machine_key(password: &str) -> AppResult<String> {
    let machine_key = generate_machine_key();
    encrypt_password(password, &machine_key)
}

/// 使用机器密钥解密密码
///
/// 结合了密钥生成和密码解密两个步骤的便捷函数
///
/// # 参数
/// * `encrypted_password` - 需要解密的密文密码
///
/// # 返回值
/// 返回解密后的明文密码，或包含错误信息的AppError
pub fn decrypt_password_with_machine_key(encrypted_password: &str) -> AppResult<String> {
    let machine_key = generate_machine_key();
    decrypt_password(encrypted_password, &machine_key)
}

/// 解密配置中的密码
///
/// 专门用于解密配置文件中存储的密码，会将`CryptoError`转换为`PasswordDecryptionError`
///
/// # 参数
/// * `encrypted_password` - 需要解密的密文密码
///
/// # 返回值
/// 返回解密后的明文密码，或包含错误信息的AppError
pub fn decrypt_config_password(encrypted_password: &str) -> AppResult<String> {
    let machine_key = generate_machine_key();
    decrypt_password(encrypted_password, &machine_key).map_err(|e| {
        match e {
            // 将加密错误转换为密码解密错误，隐藏内部细节
            AppError::CryptoError(internal_msg) => AppError::PasswordDecryptionError {
                internal_msg,
                user_msg: "密码解密失败，请重新输入密码".to_string(),
            },
            _ => e,
        }
    })
}

/// 生成加密密码的统一函数，供配置模块使用
///
/// # 参数
/// * `password` - 需要加密的明文密码
///
/// # 返回值
/// 返回加密后的密码字符串，如果加密失败则返回空字符串
pub fn generate_encrypted_password(password: &str) -> String {
    if !password.is_empty() {
        encrypt_password_with_machine_key(password).unwrap_or_else(|_| String::new())
    } else {
        String::new()
    }
}

/// 统一处理密码解密错误的函数
///
/// # 参数
/// * `result` - 解密结果
/// * `event_bus` - 事件总线，用于发送错误通知
///
/// # 返回值
/// 返回解密后的密码，如果解密失败则返回空字符串
pub fn handle_password_decryption_error_with_default(
    result: Result<String, AppError>,
    event_bus: &EventBus,
) -> String {
    handle_password_decryption_error(result, event_bus).unwrap_or_else(|_| String::new())
}

/// 统一处理密码解密错误
///
/// 用于在GUI和核心服务中统一处理密码解密失败的情况
/// 返回解密后的密码，如果解密失败则通过事件通知并返回错误
///
/// # 参数
/// * `decrypt_result` - 解密操作的结果
/// * `event_bus` - 事件总线，用于发送错误通知
///
/// # 返回值
/// 返回解密后的密码，如果解密失败则返回包含错误信息的AppError
pub fn handle_password_decryption_error(
    decrypt_result: Result<String, AppError>,
    event_bus: &EventBus,
) -> Result<String, AppError> {
    match decrypt_result {
        Ok(password) => Ok(password),
        Err(e) => match &e {
            AppError::PasswordDecryptionError {
                internal_msg,
                user_msg,
            } => {
                notify_network_status_checked(
                    event_bus,
                    CampusNetworkStatus::NotLoggedIn,
                    WanStatus::CheckFailed,
                    internal_msg,
                );

                notify_login_attempted(event_bus, false, user_msg, 0.0);

                Err(e)
            }
            _ => {
                let internal_error = format!("解密密码失败: {}", e);
                let user_message = "密码解密失败，请重新输入密码";

                notify_network_status_checked(
                    event_bus,
                    CampusNetworkStatus::NotLoggedIn,
                    WanStatus::CheckFailed,
                    &internal_error,
                );

                notify_login_attempted(event_bus, false, user_message, 0.0);

                Err(AppError::PasswordDecryptionError {
                    internal_msg: internal_error,
                    user_msg: user_message.to_string(),
                })
            }
        },
    }
}
