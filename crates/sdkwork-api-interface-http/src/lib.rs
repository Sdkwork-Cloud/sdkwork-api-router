mod assistants_handlers;
mod assistants_stateless_handlers;
mod batches_handlers;
mod batches_stateless_handlers;
mod chat_completion_handlers;
mod chat_completion_stateless_handlers;
mod compat_anthropic;
mod compat_anthropic_handlers;
mod compat_gemini;
mod compat_gemini_handlers;
mod compat_streaming;
mod conversation_handlers;
mod conversation_stateless_handlers;
mod eval_handlers;
mod eval_stateless_handlers;
mod fine_tuning_handlers;
mod fine_tuning_stateless_handlers;
mod gateway_auth;
mod gateway_docs;
mod gateway_prelude;
mod gateway_response_helpers;
mod gateway_router_builders;
mod gateway_router_common;
mod gateway_state;
mod gateway_stateful_route_groups;
mod gateway_stateful_router;
mod gateway_stateless_route_groups;
mod gateway_stateless_router;
mod gateway_usage;
mod inference_handlers;
mod inference_stateless_handlers;
mod models_handlers;
mod models_stateless_handlers;
mod multipart_parsers;
mod realtime_handlers;
mod realtime_stateless_handlers;
mod response_handlers;
mod response_stateless_handlers;
mod stateless_gateway;
mod stateless_relay;
mod storage_handlers;
mod storage_stateless_handlers;
mod thread_handlers;
mod thread_stateless_handlers;
mod vector_store_handlers;
mod vector_store_stateless_handlers;
mod video_handlers;
mod video_stateless_handlers;
mod webhooks_handlers;
mod webhooks_stateless_handlers;

use gateway_prelude::*;
pub use gateway_router_builders::{
    gateway_router_with_pool, gateway_router_with_pool_and_master_key,
    gateway_router_with_pool_and_secret_manager, gateway_router_with_store,
    gateway_router_with_store_and_secret_manager,
};
pub use gateway_state::GatewayApiState;
pub use stateless_gateway::{StatelessGatewayConfig, StatelessGatewayUpstream};

pub fn gateway_router() -> Router {
    gateway_router_with_stateless_config(StatelessGatewayConfig::default())
}

pub fn gateway_router_with_stateless_config(config: StatelessGatewayConfig) -> Router {
    build_stateless_gateway_router(config)
}

pub fn gateway_router_with_state(state: GatewayApiState) -> Router {
    build_stateful_gateway_router(state)
}
