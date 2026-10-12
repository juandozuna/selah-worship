use selah_application::OutputPublisher;
use std::ffi::{c_char, c_void, CString};

#[repr(C)]
struct SenderConfig {
    name: *const c_char,
    groups: *const c_char,
    clock_video: bool,
    clock_audio: bool,
}
#[repr(C)]
struct VideoFrame {
    width: i32,
    height: i32,
    fourcc: u32,
    rate_n: i32,
    rate_d: i32,
    aspect: f32,
    format: i32,
    timecode: i64,
    data: *const u8,
    stride: i32,
    metadata: *const c_char,
    timestamp: i64,
}
type Create = unsafe extern "C" fn(*const SenderConfig) -> *mut c_void;
type Send = unsafe extern "C" fn(*mut c_void, *const VideoFrame);
type Destroy = unsafe extern "C" fn(*mut c_void);
type Connections = unsafe extern "C" fn(*mut c_void, u32) -> i32;

struct Library(*mut c_void);
#[cfg(unix)]
#[link(name = "System")]
unsafe extern "C" {
    fn dlopen(path: *const c_char, flags: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> i32;
}
#[cfg(windows)]
#[link(name = "Kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(path: *const u16) -> *mut c_void;
    fn GetProcAddress(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn FreeLibrary(handle: *mut c_void) -> i32;
}
impl Library {
    fn open(path: &str) -> Option<Self> {
        #[cfg(unix)]
        let h = unsafe { dlopen(CString::new(path).ok()?.as_ptr(), 2) };
        #[cfg(windows)]
        let h = unsafe {
            LoadLibraryW(
                format!("{path}\0")
                    .encode_utf16()
                    .collect::<Vec<_>>()
                    .as_ptr(),
            )
        };
        if h.is_null() {
            None
        } else {
            Some(Self(h))
        }
    }
    fn symbol(&self, name: &str) -> Result<*mut c_void, String> {
        let name = CString::new(name).map_err(|_| "Invalid symbol")?;
        #[cfg(unix)]
        let symbol = unsafe { dlsym(self.0, name.as_ptr()) };
        #[cfg(windows)]
        let symbol = unsafe { GetProcAddress(self.0, name.as_ptr()) };
        if symbol.is_null() {
            Err("Installed NDI runtime is missing required SDK exports".into())
        } else {
            Ok(symbol)
        }
    }
}
impl Drop for Library {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            dlclose(self.0);
        };
        #[cfg(windows)]
        unsafe {
            FreeLibrary(self.0);
        };
    }
}

pub struct NdiPublisher {
    sender: *mut c_void,
    send: Send,
    destroy: Destroy,
    destroy_runtime: unsafe extern "C" fn(),
    connections: Connections,
    _library: Library,
    _name: CString,
}
// The handle is moved to and used exclusively by the render worker; synchronous sends finish before frame memory is reused.
unsafe impl std::marker::Send for NdiPublisher {}
impl NdiPublisher {
    pub fn new(name: &str) -> Result<Self, String> {
        let mut paths = Vec::new();
        if let Ok(dir) = std::env::var("NDI_RUNTIME_DIR_V6") {
            paths.push(format!("{dir}/Processing.NDI.Lib.x64.dll"));
        }
        if let Ok(dir) = std::env::var("NDI_RUNTIME_DIR_V5") {
            paths.push(format!("{dir}/Processing.NDI.Lib.x64.dll"));
        }
        #[cfg(target_os = "macos")]
        paths.extend(
            [
                "/Library/NDI SDK for Apple/lib/macOS/libndi.dylib",
                "/usr/local/lib/libndi.dylib",
                "libndi.dylib",
            ]
            .map(str::to_string),
        );
        #[cfg(windows)]
        paths.push("Processing.NDI.Lib.x64.dll".into());
        let library = paths
            .iter()
            .find_map(|p| Library::open(p))
            .ok_or("NDI runtime not found. Install the NDI SDK/runtime and restart Selah.")?;
        let initialize: unsafe extern "C" fn() -> bool =
            unsafe { std::mem::transmute(library.symbol("NDIlib_initialize")?) };
        let destroy_runtime: unsafe extern "C" fn() =
            unsafe { std::mem::transmute(library.symbol("NDIlib_destroy")?) };
        let create: Create = unsafe { std::mem::transmute(library.symbol("NDIlib_send_create")?) };
        let send = unsafe {
            std::mem::transmute::<*mut c_void, Send>(library.symbol("NDIlib_send_send_video_v2")?)
        };
        let destroy = unsafe {
            std::mem::transmute::<*mut c_void, Destroy>(library.symbol("NDIlib_send_destroy")?)
        };
        let connections = unsafe {
            std::mem::transmute::<*mut c_void, Connections>(
                library.symbol("NDIlib_send_get_no_connections")?,
            )
        };
        let name = CString::new(name).map_err(|_| "Invalid feed name")?;
        if !unsafe { initialize() } {
            return Err("NDI cannot initialize on this computer".into());
        }
        let sender = unsafe {
            create(&SenderConfig {
                name: name.as_ptr(),
                groups: std::ptr::null(),
                clock_video: false,
                clock_audio: false,
            })
        };
        if sender.is_null() {
            unsafe { destroy_runtime() };
            return Err("NDI could not create a sender".into());
        }
        Ok(Self {
            sender,
            send,
            destroy,
            destroy_runtime,
            connections,
            _library: library,
            _name: name,
        })
    }
    pub fn receivers(&self) -> u32 {
        unsafe { (self.connections)(self.sender, 0) }.max(0) as u32
    }
}
impl OutputPublisher for NdiPublisher {
    fn publish(&mut self, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
        if rgba.len() != width as usize * height as usize * 4 {
            return Err("Invalid RGBA frame".into());
        }
        let frame = VideoFrame {
            width: width as i32,
            height: height as i32,
            fourcc: u32::from_le_bytes(*b"RGBA"),
            rate_n: 30,
            rate_d: 1,
            aspect: width as f32 / height as f32,
            format: 1,
            timecode: i64::MAX,
            data: rgba.as_ptr(),
            stride: (width * 4) as i32,
            metadata: std::ptr::null(),
            timestamp: 0,
        };
        unsafe { (self.send)(self.sender, &frame) };
        Ok(())
    }
}
impl Drop for NdiPublisher {
    fn drop(&mut self) {
        unsafe { (self.destroy)(self.sender) };
        unsafe { (self.destroy_runtime)() };
    }
}
