#[unsafe(no_mangle)]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Simple URL validation function that returns 1 for valid HTTP/HTTPS URLs, 0 otherwise
#[unsafe(no_mangle)]
pub extern "C" fn validate_url(url_ptr: *const u8, url_len: usize) -> i32 {
    if url_ptr.is_null() || url_len == 0 {
        return 0;
    }
    
    let url_bytes = unsafe { core::slice::from_raw_parts(url_ptr, url_len) };
    if let Ok(url_str) = core::str::from_utf8(url_bytes) {
        if url_str.starts_with("http://") || url_str.starts_with("https://") {
            1
        } else {
            0
        }
    } else {
        0
    }
}

// Process HTTP response and return status code (simplified)
#[unsafe(no_mangle)]
pub extern "C" fn process_response(response_ptr: *const u8, response_len: usize) -> i32 {
    if response_ptr.is_null() || response_len == 0 {
        return 0; // Invalid response
    }
    
    let response_bytes = unsafe { core::slice::from_raw_parts(response_ptr, response_len) };
    if let Ok(_response_str) = core::str::from_utf8(response_bytes) {
        // Simple processing - return 200 if we got valid UTF-8 content
        200
    } else {
        500 // Invalid UTF-8
    }
}

// Prepare HTTP GET request - validates URL and returns 1 if valid, 0 if invalid
// The host will execute the actual HTTP request asynchronously
#[unsafe(no_mangle)]
pub extern "C" fn prepare_http_get(url_ptr: *const u8, url_len: usize) -> i32 {
    if url_ptr.is_null() || url_len == 0 {
        return 0;
    }
    
    let url_bytes = unsafe { core::slice::from_raw_parts(url_ptr, url_len) };
    if let Ok(url_str) = core::str::from_utf8(url_bytes) {
        // Basic validation: must be HTTP/HTTPS and reasonable length
        if (url_str.starts_with("http://") || url_str.starts_with("https://")) 
           && url_str.len() <= 2048 {
            1 // Valid URL, proceed with request
        } else {
            0 // Invalid URL format or too long
        }
    } else {
        0 // Invalid UTF-8
    }
}

// Validate file path for reading - returns 1 if path looks safe, 0 otherwise
// The host will execute the actual file reading asynchronously
#[unsafe(no_mangle)]
pub extern "C" fn prepare_file_read(path_ptr: *const u8, path_len: usize) -> i32 {
    if path_ptr.is_null() || path_len == 0 {
        return 0;
    }
    
    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr, path_len) };
    if let Ok(path_str) = core::str::from_utf8(path_bytes) {
        // Basic safety checks
        if path_str.len() > 1024 {
            return 0; // Path too long
        }
        
        // Reject dangerous patterns
        if path_str.contains("..") || 
           path_str.starts_with("/etc/") ||
           path_str.starts_with("/sys/") ||
           path_str.starts_with("/proc/") {
            return 0; // Potentially dangerous path
        }
        
        // Must be a reasonable file extension or no extension
        if let Some(ext_pos) = path_str.rfind('.') {
            let ext = &path_str[ext_pos+1..];
            match ext {
                "txt" | "md" | "json" | "yaml" | "yml" | "toml" | "cfg" | "log" => 1,
                _ => 0 // Unsupported file type
            }
        } else {
            1 // No extension is OK
        }
    } else {
        0 // Invalid UTF-8
    }
}

// Validate file path for writing - returns 1 if path looks safe, 0 otherwise
// The host will execute the actual file writing asynchronously
#[unsafe(no_mangle)]
pub extern "C" fn prepare_file_write(path_ptr: *const u8, path_len: usize) -> i32 {
    if path_ptr.is_null() || path_len == 0 {
        return 0;
    }
    
    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr, path_len) };
    if let Ok(path_str) = core::str::from_utf8(path_bytes) {
        // Basic safety checks
        if path_str.len() > 1024 {
            return 0; // Path too long
        }
        
        // Reject dangerous patterns - more restrictive for writing
        if path_str.contains("..") || 
           path_str.starts_with("/etc/") ||
           path_str.starts_with("/sys/") ||
           path_str.starts_with("/proc/") ||
           path_str.starts_with("/usr/") ||
           path_str.starts_with("/bin/") ||
           path_str.starts_with("/sbin/") ||
           path_str.starts_with("/boot/") ||
           path_str.starts_with("/root/") {
            return 0; // Potentially dangerous path for writing
        }
        
        // Only allow writing to safe locations
        if path_str.starts_with("/tmp/") || 
           path_str.starts_with("/var/tmp/") ||
           path_str.starts_with("./") ||
           path_str.starts_with("../") == false && !path_str.starts_with("/") {
            // Must be a reasonable file extension for writing
            if let Some(ext_pos) = path_str.rfind('.') {
                let ext = &path_str[ext_pos+1..];
                match ext {
                    "txt" | "md" | "json" | "yaml" | "yml" | "toml" | "cfg" | "log" | "tmp" => 1,
                    _ => 0 // Unsupported file type for writing
                }
            } else {
                1 // No extension is OK for safe locations
            }
        } else {
            0 // Not in safe location for writing
        }
    } else {
        0 // Invalid UTF-8
    }
}

// Validate shell command text - syntax only; host enforces allow-list
#[unsafe(no_mangle)]
pub extern "C" fn prepare_shell_exec(cmd_ptr: *const u8, cmd_len: usize) -> i32 {
    if cmd_ptr.is_null() || cmd_len == 0 || cmd_len > 200 {
        return 0;
    }

    let cmd_bytes = unsafe { core::slice::from_raw_parts(cmd_ptr, cmd_len) };
    if let Ok(cmd_str) = core::str::from_utf8(cmd_bytes) {
        // Deny dangerous characters or constructs
        if cmd_str.contains('|')
            || cmd_str.contains('>')
            || cmd_str.contains('<')
            || cmd_str.contains(';')
            || cmd_str.contains('&')
            || cmd_str.contains('`')
            || cmd_str.contains('\n')
        {
            return 0;
        }
        // Basic allowed charset check (letters, numbers, space, dot, slash, dash, underscore)
        if !cmd_str.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '.' | '/' | '-' | '_' )) {
            return 0;
        }
        1
    } else {
        0
    }
}

// Validate recommendation task input (length and charset)
#[unsafe(no_mangle)]
pub extern "C" fn prepare_recommend_mcps(task_ptr: *const u8, task_len: usize) -> i32 {
    if task_ptr.is_null() || task_len == 0 || task_len > 500 {
        return 0;
    }
    let bytes = unsafe { core::slice::from_raw_parts(task_ptr, task_len) };
    if let Ok(s) = core::str::from_utf8(bytes) {
        // Permit common punctuation
        if !s.chars().all(|ch| ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() || matches!(ch, '.' | ',' | ':' | ';' | '!' | '?' | '-' | '_' | '/' | '(' | ')' | '"' | '\'')) {
            return 0;
        }
        1
    } else {
        0
    }
}
