use std::fmt;

const MAGIC: &[u8] = b"PARLOVOICE1";

#[derive(Debug)]
pub enum CryptError {
    Unsupported,
    NotSealed,
    Failed,
}

impl fmt::Display for CryptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptError::Unsupported => {
                write!(formatter, "sealing a sample needs Windows")
            }
            CryptError::NotSealed => {
                write!(
                    formatter,
                    "this sample is not sealed, so it cannot be opened"
                )
            }
            CryptError::Failed => write!(
                formatter,
                "Windows refused to unseal this sample; another account owns it"
            ),
        }
    }
}

impl std::error::Error for CryptError {}

pub fn sealed(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}

pub fn seal(bytes: &[u8]) -> Result<Vec<u8>, CryptError> {
    let mut sealed = MAGIC.to_vec();
    sealed.extend_from_slice(&platform::protect(bytes)?);
    Ok(sealed)
}

pub fn open(bytes: &[u8]) -> Result<Vec<u8>, CryptError> {
    let Some(sealed) = bytes.strip_prefix(MAGIC) else {
        return Err(CryptError::NotSealed);
    };
    platform::unprotect(sealed)
}

#[cfg(windows)]
mod platform {
    use std::ffi::c_void;

    use super::CryptError;

    const UI_FORBIDDEN: u32 = 0x1;

    #[repr(C)]
    struct Blob {
        size: u32,
        data: *mut u8,
    }

    #[link(name = "crypt32")]
    extern "system" {
        fn CryptProtectData(
            input: *const Blob,
            description: *const u16,
            entropy: *const Blob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            output: *mut Blob,
        ) -> i32;
        fn CryptUnprotectData(
            input: *const Blob,
            description: *mut *mut u16,
            entropy: *const Blob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            output: *mut Blob,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(memory: *mut c_void) -> *mut c_void;
    }

    fn take(output: Blob) -> Vec<u8> {
        if output.data.is_null() || output.size == 0 {
            return Vec::new();
        }
        let bytes =
            unsafe { std::slice::from_raw_parts(output.data, output.size as usize) }.to_vec();
        unsafe { LocalFree(output.data as *mut c_void) };
        bytes
    }

    pub fn protect(bytes: &[u8]) -> Result<Vec<u8>, CryptError> {
        let input = Blob {
            size: bytes.len() as u32,
            data: bytes.as_ptr() as *mut u8,
        };
        let mut output = Blob {
            size: 0,
            data: std::ptr::null_mut(),
        };
        let done = unsafe {
            CryptProtectData(
                &input,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                UI_FORBIDDEN,
                &mut output,
            )
        };
        if done == 0 {
            return Err(CryptError::Failed);
        }
        Ok(take(output))
    }

    pub fn unprotect(bytes: &[u8]) -> Result<Vec<u8>, CryptError> {
        let input = Blob {
            size: bytes.len() as u32,
            data: bytes.as_ptr() as *mut u8,
        };
        let mut output = Blob {
            size: 0,
            data: std::ptr::null_mut(),
        };
        let done = unsafe {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                UI_FORBIDDEN,
                &mut output,
            )
        };
        if done == 0 {
            return Err(CryptError::Failed);
        }
        Ok(take(output))
    }
}

#[cfg(not(windows))]
mod platform {
    use super::CryptError;

    pub fn protect(_bytes: &[u8]) -> Result<Vec<u8>, CryptError> {
        Err(CryptError::Unsupported)
    }

    pub fn unprotect(_bytes: &[u8]) -> Result<Vec<u8>, CryptError> {
        Err(CryptError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn a_sealed_sample_opens_again() {
        let sealed = seal(b"RIFFwav").unwrap();
        assert!(sealed.starts_with(MAGIC));
        assert_eq!(open(&sealed).unwrap(), b"RIFFwav");
    }

    #[test]
    fn a_plain_sample_is_not_mistaken_for_a_sealed_one() {
        assert!(!sealed(b"RIFFwav"));
        assert!(matches!(open(b"RIFFwav"), Err(CryptError::NotSealed)));
    }

    #[cfg(windows)]
    #[test]
    fn a_sealed_sample_does_not_carry_its_plain_text() {
        let sealed = seal(b"RIFFwav").unwrap();
        assert!(!sealed.windows(7).any(|window| window == b"RIFFwav"));
    }
}
