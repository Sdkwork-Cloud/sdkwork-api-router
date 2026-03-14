use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SDKWORK_EXTENSION_ABI_VERSION: u32 = 1;

pub const SDKWORK_EXTENSION_ABI_VERSION_SYMBOL: &[u8] = b"sdkwork_extension_abi_version\0";
pub const SDKWORK_EXTENSION_MANIFEST_JSON_SYMBOL: &[u8] = b"sdkwork_extension_manifest_json\0";
pub const SDKWORK_EXTENSION_PROVIDER_EXECUTE_JSON_SYMBOL: &[u8] =
    b"sdkwork_extension_provider_execute_json\0";
pub const SDKWORK_EXTENSION_FREE_STRING_SYMBOL: &[u8] = b"sdkwork_extension_free_string\0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderInvocation {
    pub operation: String,
    pub api_key: String,
    pub base_url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path_params: Vec<String>,
    pub body: Value,
    #[serde(default)]
    pub expects_stream: bool,
}

impl ProviderInvocation {
    pub fn new(
        operation: impl Into<String>,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        path_params: Vec<String>,
        body: Value,
        expects_stream: bool,
    ) -> Self {
        Self {
            operation: operation.into(),
            api_key: api_key.into(),
            base_url: base_url.into(),
            path_params,
            body,
            expects_stream,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderInvocationResult {
    Json {
        body: Value,
    },
    Unsupported {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    Error {
        message: String,
    },
}

impl ProviderInvocationResult {
    pub fn json(body: Value) -> Self {
        Self::Json { body }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported {
            message: Some(message.into()),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }
}

pub fn into_raw_c_string(value: impl Into<String>) -> *mut c_char {
    CString::new(value.into()).expect("c string").into_raw()
}

/// # Safety
///
/// The pointer must be either null or a valid, NUL-terminated C string.
pub unsafe fn from_raw_c_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
}

/// # Safety
///
/// The pointer must have been allocated by `CString::into_raw` and not yet freed.
pub unsafe fn free_raw_c_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    let _ = CString::from_raw(ptr);
}
