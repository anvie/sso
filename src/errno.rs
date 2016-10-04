// Derived from http error codes.
// https://en.wikipedia.org/wiki/List_of_HTTP_status_codes

pub const BAD_REQQUEST:i32 = 400; // Bad Request
pub const UNAUTHORIZED:i32 = 401; // Unauthorized
pub const NOT_FOUND:i32 = 404; // Not found
pub const INVALID_TOKEN:i32 = 498;
pub const INTERNAL_SERVER_ERROR:i32 = 500;

pub static BAD_REQUEST_STR:&'static str = "Bad request";
pub static UNAUTHORIZED_STR:&'static str = "Access denied";
pub static NOT_FOUND_STR:&'static str = "Not found";
pub static INVALID_TOKEN_STR:&'static str = "Invalid token";
