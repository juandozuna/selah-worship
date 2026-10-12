use selah_application::CredentialStore;
pub struct SystemCredentials;
fn valid(provider: &str) -> Result<(), String> {
    if !["brave", "openai", "anthropic", "gemini", "compatible"].contains(&provider) {
        return Err("Unknown credential provider".into());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
mod platform {
    use std::ffi::c_void;
    #[link(name = "Security", kind = "framework")]
    unsafe extern "C" {
        fn SecKeychainFindGenericPassword(
            keychain: *const c_void,
            service_len: u32,
            service: *const u8,
            account_len: u32,
            account: *const u8,
            len: *mut u32,
            data: *mut *mut c_void,
            item: *mut *mut c_void,
        ) -> i32;
        fn SecKeychainAddGenericPassword(
            keychain: *const c_void,
            service_len: u32,
            service: *const u8,
            account_len: u32,
            account: *const u8,
            len: u32,
            data: *const u8,
            item: *mut *mut c_void,
        ) -> i32;
        fn SecKeychainItemModifyAttributesAndData(
            item: *const c_void,
            attrs: *const c_void,
            len: u32,
            data: *const u8,
        ) -> i32;
        fn SecKeychainItemFreeContent(attrs: *const c_void, data: *mut c_void) -> i32;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFRelease(object: *const c_void);
    }
    pub fn get(provider: &str) -> Result<String, String> {
        let service = b"com.selah.worship";
        let mut len = 0;
        let mut data = std::ptr::null_mut();
        let code = unsafe {
            SecKeychainFindGenericPassword(
                std::ptr::null(),
                service.len() as u32,
                service.as_ptr(),
                provider.len() as u32,
                provider.as_ptr(),
                &mut len,
                &mut data,
                std::ptr::null_mut(),
            )
        };
        if code != 0 {
            return Err("Add your API key in Settings".into());
        }
        let bytes = unsafe { std::slice::from_raw_parts(data as *const u8, len as usize).to_vec() };
        unsafe { SecKeychainItemFreeContent(std::ptr::null(), data) };
        String::from_utf8(bytes).map_err(|_| "Invalid stored credential".into())
    }
    pub fn set(provider: &str, value: &str) -> Result<(), String> {
        let service = b"com.selah.worship";
        let mut item = std::ptr::null_mut();
        let code = unsafe {
            SecKeychainFindGenericPassword(
                std::ptr::null(),
                service.len() as u32,
                service.as_ptr(),
                provider.len() as u32,
                provider.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut item,
            )
        };
        let code = if code == 0 {
            let c = unsafe {
                SecKeychainItemModifyAttributesAndData(
                    item,
                    std::ptr::null(),
                    value.len() as u32,
                    value.as_ptr(),
                )
            };
            unsafe { CFRelease(item) };
            c
        } else {
            unsafe {
                SecKeychainAddGenericPassword(
                    std::ptr::null(),
                    service.len() as u32,
                    service.as_ptr(),
                    provider.len() as u32,
                    provider.as_ptr(),
                    value.len() as u32,
                    value.as_ptr(),
                    std::ptr::null_mut(),
                )
            }
        };
        if code != 0 {
            Err(format!("Keychain could not save the key ({code})"))
        } else {
            Ok(())
        }
    }
}
#[cfg(target_os = "windows")]
mod platform {
    use std::ffi::c_void;
    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    #[repr(C)]
    struct Credential {
        flags: u32,
        kind: u32,
        target: *mut u16,
        comment: *mut u16,
        time: FileTime,
        size: u32,
        blob: *mut u8,
        persist: u32,
        count: u32,
        attrs: *mut c_void,
        alias: *mut u16,
        user: *mut u16,
    }
    #[link(name = "Advapi32")]
    unsafe extern "system" {
        fn CredReadW(target: *const u16, kind: u32, flags: u32, out: *mut *mut Credential) -> i32;
        fn CredWriteW(credential: *const Credential, flags: u32) -> i32;
        fn CredFree(ptr: *mut c_void);
    }
    fn target(p: &str) -> Vec<u16> {
        format!("com.selah.worship/{p}\0").encode_utf16().collect()
    }
    pub fn get(p: &str) -> Result<String, String> {
        let name = target(p);
        let mut ptr = std::ptr::null_mut();
        if unsafe { CredReadW(name.as_ptr(), 1, 0, &mut ptr) } == 0 {
            return Err("Add your API key in Settings".into());
        }
        let b = unsafe { std::slice::from_raw_parts((*ptr).blob, (*ptr).size as usize).to_vec() };
        unsafe { CredFree(ptr.cast()) };
        String::from_utf8(b).map_err(|_| "Invalid stored credential".into())
    }
    pub fn set(p: &str, v: &str) -> Result<(), String> {
        let mut name = target(p);
        let c = Credential {
            flags: 0,
            kind: 1,
            target: name.as_mut_ptr(),
            comment: std::ptr::null_mut(),
            time: FileTime { low: 0, high: 0 },
            size: v.len() as u32,
            blob: v.as_ptr() as *mut u8,
            persist: 2,
            count: 0,
            attrs: std::ptr::null_mut(),
            alias: std::ptr::null_mut(),
            user: std::ptr::null_mut(),
        };
        if unsafe { CredWriteW(&c, 0) } == 0 {
            Err("Windows Credential Manager could not save the key".into())
        } else {
            Ok(())
        }
    }
}
impl CredentialStore for SystemCredentials {
    fn get(&self, p: &str) -> Result<String, String> {
        valid(p)?;
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            platform::get(p)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err("Credential store supports macOS and Windows".into())
        }
    }
    fn set(&self, p: &str, v: &str) -> Result<(), String> {
        valid(p)?;
        if v.is_empty() || v.len() > 4096 || v.contains(['\n', '\r', '\0']) {
            return Err("Invalid API key".into());
        }
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            platform::set(p, v)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err("Credential store supports macOS and Windows".into())
        }
    }
}
