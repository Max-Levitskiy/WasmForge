// Minimal AssemblyScript exports without runtime allocations/imports
// Reads input directly from memory via load<u8>(ptr + i)

export function add(a: i32, b: i32): i32 {
  return a + b;
}

function startsWithHttp(ptr: i32, len: i32): i32 {
  // "http://"
  if (len < 7) return 0;
  if (load<u8>(ptr + 0) != 104) return 0; // h
  if (load<u8>(ptr + 1) != 116) return 0; // t
  if (load<u8>(ptr + 2) != 116) return 0; // t
  if (load<u8>(ptr + 3) != 112) return 0; // p
  if (load<u8>(ptr + 4) != 58) return 0;  // :
  if (load<u8>(ptr + 5) != 47) return 0;  // /
  if (load<u8>(ptr + 6) != 47) return 0;  // /
  return 1;
}

function startsWithHttps(ptr: i32, len: i32): i32 {
  // "https://"
  if (len < 8) return 0;
  if (load<u8>(ptr + 0) != 104) return 0; // h
  if (load<u8>(ptr + 1) != 116) return 0; // t
  if (load<u8>(ptr + 2) != 116) return 0; // t
  if (load<u8>(ptr + 3) != 112) return 0; // p
  if (load<u8>(ptr + 4) != 115) return 0; // s
  if (load<u8>(ptr + 5) != 58) return 0;  // :
  if (load<u8>(ptr + 6) != 47) return 0;  // /
  if (load<u8>(ptr + 7) != 47) return 0;  // /
  return 1;
}

export function validate_url(ptr: i32, len: i32): i32 {
  if (ptr == 0 || len <= 0) return 0;
  if (startsWithHttp(ptr, len)) return 1;
  if (startsWithHttps(ptr, len)) return 1;
  return 0;
}

export function prepare_http_get(ptr: i32, len: i32): i32 {
  if (ptr == 0 || len <= 0) return 0;
  if (len > 2048) return 0;
  if (startsWithHttp(ptr, len)) return 1;
  if (startsWithHttps(ptr, len)) return 1;
  return 0;
}
