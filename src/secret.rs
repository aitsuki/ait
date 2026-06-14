use crate::error::{AppError, Result};

pub struct SecretStore {
    purpose: String,
}

impl SecretStore {
    pub fn new(purpose: impl Into<String>) -> Self {
        Self {
            purpose: purpose.into(),
        }
    }

    #[cfg(not(windows))]
    pub fn protect(&self, _plain: &str) -> Result<String> {
        Err(AppError::Secret("DPAPI 仅支持 Windows".to_string()))
    }

    #[cfg(not(windows))]
    pub fn unprotect(&self, _encrypted: &str) -> Result<String> {
        Err(AppError::Secret("DPAPI 仅支持 Windows".to_string()))
    }
}

#[cfg(windows)]
impl SecretStore {
    pub fn protect(&self, plain: &str) -> Result<String> {
        use windows::Win32::Foundation::{HLOCAL, LocalFree};
        use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptProtectData};
        use windows::core::PCWSTR;

        let mut input = plain.as_bytes().to_vec();
        let mut in_blob = CRYPT_INTEGER_BLOB {
            cbData: input.len() as u32,
            pbData: input.as_mut_ptr(),
        };
        let description: Vec<u16> = self.purpose.encode_utf16().chain(Some(0)).collect();
        let mut out_blob = CRYPT_INTEGER_BLOB::default();

        unsafe {
            CryptProtectData(
                &mut in_blob,
                PCWSTR(description.as_ptr()),
                None,
                None,
                None,
                0,
                &mut out_blob,
            )
            .map_err(|err| AppError::Secret(format!("DPAPI 加密失败: {err}")))?;

            let bytes =
                std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec();
            let _ = LocalFree(Some(HLOCAL(out_blob.pbData as _)));
            Ok(base64_encode(&bytes))
        }
    }

    pub fn unprotect(&self, encrypted: &str) -> Result<String> {
        use windows::Win32::Foundation::{HLOCAL, LocalFree};
        use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptUnprotectData};

        let mut input = base64_decode(encrypted)?;
        let mut in_blob = CRYPT_INTEGER_BLOB {
            cbData: input.len() as u32,
            pbData: input.as_mut_ptr(),
        };
        let mut out_blob = CRYPT_INTEGER_BLOB::default();

        unsafe {
            CryptUnprotectData(&mut in_blob, None, None, None, None, 0, &mut out_blob)
                .map_err(|err| AppError::Secret(format!("DPAPI 解密失败: {err}")))?;

            let bytes =
                std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec();
            let _ = LocalFree(Some(HLOCAL(out_blob.pbData as _)));
            String::from_utf8(bytes)
                .map_err(|err| AppError::Secret(format!("密钥不是 UTF-8: {err}")))
        }
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(b2 & 0b0011_1111) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let cleaned = input.trim().as_bytes();
    if cleaned.len() % 4 != 0 {
        return Err(AppError::Secret("无效的 base64 密文长度".to_string()));
    }
    for chunk in cleaned.chunks(4) {
        let vals: Vec<u8> = chunk
            .iter()
            .map(|b| match b {
                b'A'..=b'Z' => b - b'A',
                b'a'..=b'z' => b - b'a' + 26,
                b'0'..=b'9' => b - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                b'=' => 64,
                _ => 255,
            })
            .collect();
        if vals.iter().any(|v| *v == 255) {
            return Err(AppError::Secret("无效的 base64 字符".to_string()));
        }
        bytes.push((vals[0] << 2) | (vals[1] >> 4));
        if vals[2] != 64 {
            bytes.push((vals[1] << 4) | (vals[2] >> 2));
        }
        if vals[3] != 64 {
            bytes.push((vals[2] << 6) | vals[3]);
        }
    }
    Ok(bytes)
}
