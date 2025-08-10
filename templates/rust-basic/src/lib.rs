#[unsafe(no_mangle)]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[unsafe(no_mangle)]
pub extern "C" fn validate_url(url_ptr: *const u8, url_len: usize) -> i32 {
    if url_ptr.is_null() || url_len == 0 {
        return 0;
    }
    let url_bytes = unsafe { core::slice::from_raw_parts(url_ptr, url_len) };
    if let Ok(url_str) = core::str::from_utf8(url_bytes) {
        if url_str.starts_with("http://") || url_str.starts_with("https://") { 1 } else { 0 }
    } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn process_response(response_ptr: *const u8, response_len: usize) -> i32 {
    if response_ptr.is_null() || response_len == 0 { return 0; }
    let response_bytes = unsafe { core::slice::from_raw_parts(response_ptr, response_len) };
    if core::str::from_utf8(response_bytes).is_ok() { 200 } else { 500 }
}

#[unsafe(no_mangle)]
pub extern "C" fn prepare_http_get(url_ptr: *const u8, url_len: usize) -> i32 {
    if url_ptr.is_null() || url_len == 0 { return 0; }
    let url_bytes = unsafe { core::slice::from_raw_parts(url_ptr, url_len) };
    if let Ok(url_str) = core::str::from_utf8(url_bytes) {
        if (url_str.starts_with("http://") || url_str.starts_with("https://")) && url_str.len() <= 2048 { 1 } else { 0 }
    } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn prepare_file_read(path_ptr: *const u8, path_len: usize) -> i32 {
    if path_ptr.is_null() || path_len == 0 { return 0; }
    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr, path_len) };
    if let Ok(path_str) = core::str::from_utf8(path_bytes) {
        if path_str.len() > 1024 { return 0; }
        if path_str.contains("..") || path_str.starts_with("/etc/") || path_str.starts_with("/sys/") || path_str.starts_with("/proc/") { return 0; }
        if let Some(ext_pos) = path_str.rfind('.') {
            match &path_str[ext_pos+1..] { "txt"|"md"|"json"|"yaml"|"yml"|"toml"|"cfg"|"log" => 1, _ => 0 }
        } else { 1 }
    } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn prepare_file_write(path_ptr: *const u8, path_len: usize) -> i32 {
    if path_ptr.is_null() || path_len == 0 { return 0; }
    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr, path_len) };
    if let Ok(path_str) = core::str::from_utf8(path_bytes) {
        if path_str.len() > 1024 { return 0; }
        if path_str.contains("..") || path_str.starts_with("/etc/") || path_str.starts_with("/sys/") || path_str.starts_with("/proc/") || path_str.starts_with("/usr/") || path_str.starts_with("/bin/") || path_str.starts_with("/sbin/") || path_str.starts_with("/boot/") || path_str.starts_with("/root/") { return 0; }
        if path_str.starts_with("/tmp/") || path_str.starts_with("/var/tmp/") || path_str.starts_with("./") || (!path_str.starts_with("/") && !path_str.starts_with("../")) {
            if let Some(ext_pos) = path_str.rfind('.') {
                match &path_str[ext_pos+1..] { "txt"|"md"|"json"|"yaml"|"yml"|"toml"|"cfg"|"log"|"tmp" => 1, _ => 0 }
            } else { 1 }
        } else { 0 }
    } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn prepare_shell_exec(cmd_ptr: *const u8, cmd_len: usize) -> i32 {
    if cmd_ptr.is_null() || cmd_len == 0 || cmd_len > 200 { return 0; }
    let cmd_bytes = unsafe { core::slice::from_raw_parts(cmd_ptr, cmd_len) };
    if let Ok(cmd_str) = core::str::from_utf8(cmd_bytes) {
        if cmd_str.contains('|') || cmd_str.contains('>') || cmd_str.contains('<') || cmd_str.contains(';') || cmd_str.contains('&') || cmd_str.contains('`') || cmd_str.contains('\n') { return 0; }
        if !cmd_str.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '.' | '/' | '-' | '_' )) { return 0; }
        1
    } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn prepare_recommend_mcps(task_ptr: *const u8, task_len: usize) -> i32 {
    if task_ptr.is_null() || task_len == 0 || task_len > 500 { return 0; }
    let bytes = unsafe { core::slice::from_raw_parts(task_ptr, task_len) };
    if let Ok(s) = core::str::from_utf8(bytes) {
        if !s.chars().all(|ch| ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() || matches!(ch, '.' | ',' | ':' | ';' | '!' | '?' | '-' | '_' | '/' | '(' | ')' | '"' | '\'')) { return 0; }
        1
    } else { 0 }
}
