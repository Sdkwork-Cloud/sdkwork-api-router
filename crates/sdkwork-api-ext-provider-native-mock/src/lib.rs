use std::ffi::CString;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::raw::c_char;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use sdkwork_api_extension_abi::{
    free_raw_c_string, from_raw_c_str, into_raw_c_string, ExtensionHealthCheckResult,
    ExtensionLifecycleContext, ExtensionLifecycleResult, ProviderInvocation,
    ProviderInvocationResult, ProviderStreamInvocationResult, ProviderStreamWriter,
    SDKWORK_EXTENSION_ABI_VERSION,
};
use sdkwork_api_extension_core::{
    CapabilityDescriptor, CompatibilityLevel, ExtensionKind, ExtensionManifest,
    ExtensionPermission, ExtensionProtocol, ExtensionRuntime,
};

pub const FIXTURE_EXTENSION_ID: &str = "sdkwork.provider.native.mock";
const LIFECYCLE_LOG_ENV: &str = "SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG";
const INVOCATION_LOG_ENV: &str = "SDKWORK_NATIVE_MOCK_INVOCATION_LOG";
const JSON_DELAY_MS_ENV: &str = "SDKWORK_NATIVE_MOCK_JSON_DELAY_MS";
const STREAM_DELAY_MS_ENV: &str = "SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS";

fn manifest_json() -> &'static CString {
    static MANIFEST_JSON: OnceLock<CString> = OnceLock::new();
    MANIFEST_JSON.get_or_init(|| {
        CString::new(
            serde_json::to_string(
                &ExtensionManifest::new(
                    FIXTURE_EXTENSION_ID,
                    ExtensionKind::Provider,
                    "0.1.0",
                    ExtensionRuntime::NativeDynamic,
                )
                .with_display_name("Native Mock")
                .with_protocol(ExtensionProtocol::OpenAi)
                .with_channel_binding("sdkwork.channel.openai")
                .with_permission(ExtensionPermission::NetworkOutbound)
                .with_capability(CapabilityDescriptor::new(
                    "chat.completions.create",
                    CompatibilityLevel::Native,
                ))
                .with_capability(CapabilityDescriptor::new(
                    "chat.completions.stream",
                    CompatibilityLevel::Native,
                ))
                .with_capability(CapabilityDescriptor::new(
                    "responses.create",
                    CompatibilityLevel::Native,
                ))
                .with_capability(CapabilityDescriptor::new(
                    "responses.stream",
                    CompatibilityLevel::Native,
                ))
                .with_capability(CapabilityDescriptor::new(
                    "audio.speech.create",
                    CompatibilityLevel::Native,
                ))
                .with_capability(CapabilityDescriptor::new(
                    "files.content",
                    CompatibilityLevel::Native,
                ))
                .with_capability(CapabilityDescriptor::new(
                    "videos.content",
                    CompatibilityLevel::Native,
                )),
            )
            .expect("manifest json"),
        )
        .expect("manifest c string")
    })
}

#[no_mangle]
pub extern "C" fn sdkwork_extension_abi_version() -> u32 {
    SDKWORK_EXTENSION_ABI_VERSION
}

#[no_mangle]
pub extern "C" fn sdkwork_extension_manifest_json() -> *const c_char {
    manifest_json().as_ptr()
}

/// # Safety
///
/// `payload` must be a valid null-terminated UTF-8 JSON string for the duration
/// of this call.
#[no_mangle]
pub unsafe extern "C" fn sdkwork_extension_init_json(payload: *const c_char) -> *mut c_char {
    let result = match unsafe { from_raw_c_str(payload) }
        .and_then(|payload| serde_json::from_str::<ExtensionLifecycleContext>(&payload).ok())
    {
        Some(_) => {
            append_lifecycle_event("init");
            ExtensionLifecycleResult::success("native mock initialized")
        }
        None => ExtensionLifecycleResult::failure("invalid lifecycle payload"),
    };

    into_raw_c_string(serde_json::to_string(&result).expect("result json"))
}

/// # Safety
///
/// `payload` must be a valid null-terminated UTF-8 JSON string for the duration
/// of this call.
#[no_mangle]
pub unsafe extern "C" fn sdkwork_extension_health_check_json(
    payload: *const c_char,
) -> *mut c_char {
    let result = match unsafe { from_raw_c_str(payload) }
        .and_then(|payload| serde_json::from_str::<ExtensionLifecycleContext>(&payload).ok())
    {
        Some(_) => ExtensionHealthCheckResult::healthy("native mock healthy"),
        None => ExtensionHealthCheckResult::unhealthy("invalid lifecycle payload"),
    };

    into_raw_c_string(serde_json::to_string(&result).expect("result json"))
}

/// # Safety
///
/// `payload` must be a valid null-terminated UTF-8 JSON string for the duration
/// of this call.
#[no_mangle]
pub unsafe extern "C" fn sdkwork_extension_shutdown_json(payload: *const c_char) -> *mut c_char {
    let result = match unsafe { from_raw_c_str(payload) }
        .and_then(|payload| serde_json::from_str::<ExtensionLifecycleContext>(&payload).ok())
    {
        Some(_) => {
            append_lifecycle_event("shutdown");
            ExtensionLifecycleResult::success("native mock shut down")
        }
        None => ExtensionLifecycleResult::failure("invalid lifecycle payload"),
    };

    into_raw_c_string(serde_json::to_string(&result).expect("result json"))
}

/// # Safety
///
/// `payload` must be a valid null-terminated UTF-8 JSON string for the duration
/// of this call.
#[no_mangle]
pub unsafe extern "C" fn sdkwork_extension_provider_execute_json(
    payload: *const c_char,
) -> *mut c_char {
    let invocation = unsafe { from_raw_c_str(payload) }
        .and_then(|payload| serde_json::from_str::<ProviderInvocation>(&payload).ok());
    append_invocation_event("execute_json_start");
    sleep_for_env_delay(JSON_DELAY_MS_ENV);

    let result = match invocation {
        Some(invocation)
            if invocation.operation == "chat.completions.create" && !invocation.expects_stream =>
        {
            ProviderInvocationResult::json(serde_json::json!({
                "id": "chatcmpl_native_dynamic",
                "object": "chat.completion",
                "model": invocation.body["model"],
                "choices": [],
                "provider": "native_dynamic"
            }))
        }
        Some(invocation)
            if invocation.operation == "responses.create" && !invocation.expects_stream =>
        {
            ProviderInvocationResult::json(serde_json::json!({
                "id": "resp_native_dynamic",
                "object": "response",
                "model": invocation.body["model"],
                "output": [],
                "provider": "native_dynamic"
            }))
        }
        Some(invocation) if invocation.expects_stream => {
            ProviderInvocationResult::unsupported("stream output is not implemented in the fixture")
        }
        Some(invocation) => ProviderInvocationResult::unsupported(format!(
            "operation {} is not implemented in the fixture",
            invocation.operation
        )),
        None => ProviderInvocationResult::error("invalid invocation payload"),
    };
    append_invocation_event("execute_json_finish");

    into_raw_c_string(serde_json::to_string(&result).expect("result json"))
}

/// # Safety
///
/// `payload` must be a valid null-terminated UTF-8 JSON string and `writer`
/// must point to a valid host-owned callback table for the duration of this
/// call.
#[no_mangle]
pub unsafe extern "C" fn sdkwork_extension_provider_execute_stream_json(
    payload: *const c_char,
    writer: *const ProviderStreamWriter,
) -> *mut c_char {
    let invocation = unsafe { from_raw_c_str(payload) }
        .and_then(|payload| serde_json::from_str::<ProviderInvocation>(&payload).ok());
    let writer = unsafe { writer.as_ref() };
    append_invocation_event("execute_stream_start");

    let result = match (invocation, writer) {
        (Some(invocation), Some(writer))
            if invocation.operation == "chat.completions.create" && invocation.expects_stream =>
        {
            let content_type = "text/event-stream";
            let chunk = serde_json::json!({
                "id": "chatcmpl_native_dynamic_stream",
                "object": "chat.completion.chunk",
                "model": invocation.body["model"],
                "choices": [{
                    "index": 0,
                    "delta": {
                        "role": "assistant",
                        "content": "hello from native dynamic"
                    },
                    "finish_reason": serde_json::Value::Null
                }]
            })
            .to_string();
            let first_frame = format!("data: {chunk}\n\n");
            let done_frame = "data: [DONE]\n\n";

            if !writer.set_content_type(content_type) {
                ProviderStreamInvocationResult::error(
                    "host stream receiver closed before content type was set",
                )
            } else if !writer.write_chunk(first_frame.as_bytes()) {
                ProviderStreamInvocationResult::error(
                    "host stream receiver closed before all chunks were written",
                )
            } else {
                sleep_for_env_delay(STREAM_DELAY_MS_ENV);
                if !writer.write_chunk(done_frame.as_bytes()) {
                    ProviderStreamInvocationResult::error(
                        "host stream receiver closed before all chunks were written",
                    )
                } else {
                    ProviderStreamInvocationResult::streamed(content_type)
                }
            }
        }
        (Some(invocation), Some(writer))
            if invocation.operation == "responses.create" && invocation.expects_stream =>
        {
            let content_type = "text/event-stream";
            let chunk = serde_json::json!({
                "id": "resp_native_dynamic_stream",
                "type": "response.output_text.delta",
                "response_id": "resp_native_dynamic_stream",
                "delta": "hello from native dynamic"
            })
            .to_string();
            let first_frame = format!("data: {chunk}\n\n");
            let done_frame = "data: [DONE]\n\n";

            if !writer.set_content_type(content_type) {
                ProviderStreamInvocationResult::error(
                    "host stream receiver closed before content type was set",
                )
            } else if !writer.write_chunk(first_frame.as_bytes()) {
                ProviderStreamInvocationResult::error(
                    "host stream receiver closed before all chunks were written",
                )
            } else {
                sleep_for_env_delay(STREAM_DELAY_MS_ENV);
                if !writer.write_chunk(done_frame.as_bytes()) {
                    ProviderStreamInvocationResult::error(
                        "host stream receiver closed before all chunks were written",
                    )
                } else {
                    ProviderStreamInvocationResult::streamed(content_type)
                }
            }
        }
        (Some(invocation), Some(writer))
            if invocation.operation == "audio.speech.create" && invocation.expects_stream =>
        {
            let response_format = invocation.body["response_format"].as_str().unwrap_or("mp3");
            let content_type = match response_format {
                "wav" => "audio/wav",
                "opus" => "audio/opus",
                "aac" => "audio/aac",
                "flac" => "audio/flac",
                "pcm" => "audio/pcm",
                _ => "audio/mpeg",
            };
            let bytes = b"NATIVE-AUDIO";

            if !writer.set_content_type(content_type) {
                ProviderStreamInvocationResult::error(
                    "host stream receiver closed before content type was set",
                )
            } else {
                sleep_for_env_delay(STREAM_DELAY_MS_ENV);
                if !writer.write_chunk(bytes) {
                    ProviderStreamInvocationResult::error(
                        "host stream receiver closed before all chunks were written",
                    )
                } else {
                    ProviderStreamInvocationResult::streamed(content_type)
                }
            }
        }
        (Some(invocation), Some(writer))
            if invocation.operation == "files.content" && invocation.expects_stream =>
        {
            let content_type = "application/jsonl";
            let bytes = b"{\"source\":\"native_dynamic\"}\n";

            if !writer.set_content_type(content_type) {
                ProviderStreamInvocationResult::error(
                    "host stream receiver closed before content type was set",
                )
            } else {
                sleep_for_env_delay(STREAM_DELAY_MS_ENV);
                if !writer.write_chunk(bytes) {
                    ProviderStreamInvocationResult::error(
                        "host stream receiver closed before all chunks were written",
                    )
                } else {
                    ProviderStreamInvocationResult::streamed(content_type)
                }
            }
        }
        (Some(invocation), Some(writer))
            if invocation.operation == "videos.content" && invocation.expects_stream =>
        {
            let content_type = "video/mp4";
            let bytes = b"NATIVE-VIDEO";

            if !writer.set_content_type(content_type) {
                ProviderStreamInvocationResult::error(
                    "host stream receiver closed before content type was set",
                )
            } else {
                sleep_for_env_delay(STREAM_DELAY_MS_ENV);
                if !writer.write_chunk(bytes) {
                    ProviderStreamInvocationResult::error(
                        "host stream receiver closed before all chunks were written",
                    )
                } else {
                    ProviderStreamInvocationResult::streamed(content_type)
                }
            }
        }
        (Some(invocation), Some(_)) => ProviderStreamInvocationResult::unsupported(format!(
            "stream operation {} is not implemented in the fixture",
            invocation.operation
        )),
        (_, None) => ProviderStreamInvocationResult::error("stream writer is missing"),
        (None, Some(_)) => ProviderStreamInvocationResult::error("invalid invocation payload"),
    };
    append_invocation_event("execute_stream_finish");

    into_raw_c_string(serde_json::to_string(&result).expect("result json"))
}

/// # Safety
///
/// `ptr` must be a pointer previously returned by this library through one of
/// its string-returning ABI functions and must not be freed more than once.
#[no_mangle]
pub unsafe extern "C" fn sdkwork_extension_free_string(ptr: *mut c_char) {
    unsafe { free_raw_c_string(ptr) }
}

fn append_lifecycle_event(event: &str) {
    let Ok(path) = std::env::var(LIFECYCLE_LOG_ENV) else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "{event}");
}

fn append_invocation_event(event: &str) {
    let Ok(path) = std::env::var(INVOCATION_LOG_ENV) else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "{event}");
}

fn sleep_for_env_delay(key: &str) {
    let Ok(delay_ms) = std::env::var(key) else {
        return;
    };
    let Ok(delay_ms) = delay_ms.parse::<u64>() else {
        return;
    };
    if delay_ms == 0 {
        return;
    }
    thread::sleep(Duration::from_millis(delay_ms));
}
