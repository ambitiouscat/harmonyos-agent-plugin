use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub static STREAM_CB: Mutex<Option<extern "C" fn(chunk_data: *const c_char, event_type: u8)>> =
    Mutex::new(None);

/// Register a C callback for streaming chunks from Rust threads.
#[no_mangle]
pub extern "C" fn rust_agent_register_stream_cb(
    callback: extern "C" fn(chunk_data: *const c_char, event_type: u8),
) {
    let mut guard = STREAM_CB.lock().unwrap();
    *guard = Some(callback);
}

/// Validates network access via POSIX socket.
/// Returns true if a TCP connection to 8.8.8.8:53 can be established.
#[no_mangle]
pub extern "C" fn test_network() -> bool {
    std::net::TcpStream::connect("8.8.8.8:53").is_ok()
}

/// Validates sandbox file write permissions.
///
/// # Safety
/// `dir` must be a valid null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn test_file(dir: *const c_char) -> bool {
    use std::ffi::CStr;
    if dir.is_null() {
        return false;
    }
    let dir_str = match CStr::from_ptr(dir).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    let path = std::path::Path::new(dir_str).join("_hmos_agent_test_.tmp");
    if std::fs::write(&path, b"test").is_err() {
        return false;
    }
    std::fs::remove_file(&path).is_ok()
}

/// Spawns a background thread that sends ContentPart JSON deltas via the
/// stream callback. Phase 2: outputs structured JSON instead of raw chars.
/// event_type 0 = data chunk, 1 = normal completion.
pub fn start_stream_sim(chunks: u32, interval_ms: u64) {
    thread::spawn(move || {
        let cb = {
            let guard = STREAM_CB.lock().unwrap();
            *guard
        };

        let Some(callback) = cb else {
            return;
        };

        let text = "Hello World";

        for i in 0..chunks {
            let ch_idx = (i as usize) % text.len();
            let ch = text.chars().nth(ch_idx).unwrap_or('?');
            let json = format!(r#"{{"type":"text","text":"{}"}}"#, ch);
            let s = CString::new(json).unwrap();
            callback(s.as_ptr(), 0);
            thread::sleep(Duration::from_millis(interval_ms));
        }

        let done = CString::new("").unwrap();
        callback(done.as_ptr(), 1);
    });
}
