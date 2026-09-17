//! 设备凭据记录的本地保护。
//!
//! Windows 上使用 DPAPI（`CryptProtectData`）的用户作用域：密文与当前 Windows 账户绑定，
//! 凭据文件被复制到其他账户或其他机器后无法解读。
//! 桌面客户端只发布 Windows 版本，其他平台的构建（CI 校验、开发）不做本地保护。

#[cfg(windows)]
use windows::Win32::Security::Cryptography::CRYPT_INTEGER_BLOB;

#[cfg(windows)]
pub fn protect(plaintext: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN};

    let input = input_blob(plaintext)?;
    let mut output = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptProtectData(
            &input,
            windows::core::PCWSTR::null(),
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
        .map_err(|error| format!("credential record cannot be protected: {error}"))?;
        Ok(take_output(output))
    }
}

#[cfg(windows)]
pub fn unprotect(protected: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN};

    let input = input_blob(protected)?;
    let mut output = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptUnprotectData(
            &input,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
        .map_err(|error| {
            format!(
                "credential record cannot be decrypted for the current Windows account: {error}"
            )
        })?;
        Ok(take_output(output))
    }
}

#[cfg(windows)]
fn input_blob(value: &[u8]) -> Result<CRYPT_INTEGER_BLOB, String> {
    let length = u32::try_from(value.len())
        .map_err(|_| "credential record is too large to protect".to_owned())?;
    Ok(CRYPT_INTEGER_BLOB {
        cbData: length,
        pbData: value.as_ptr().cast_mut(),
    })
}

#[cfg(windows)]
fn take_output(blob: CRYPT_INTEGER_BLOB) -> Vec<u8> {
    use windows::Win32::Foundation::{LocalFree, HLOCAL};

    // SAFETY: 缓冲区由 DPAPI 分配，`cbData` 与 `pbData` 来自同一次调用，复制后立即用 LocalFree 释放。
    let value = unsafe { std::slice::from_raw_parts(blob.pbData, blob.cbData as usize).to_vec() };
    unsafe {
        let _ = LocalFree(Some(HLOCAL(blob.pbData.cast())));
    }
    value
}

#[cfg(not(windows))]
pub fn protect(plaintext: &[u8]) -> Result<Vec<u8>, String> {
    Ok(plaintext.to_vec())
}

#[cfg(not(windows))]
pub fn unprotect(protected: &[u8]) -> Result<Vec<u8>, String> {
    Ok(protected.to_vec())
}
