use super::*;

mod channel;
mod credential;
mod model;
mod provider;
mod provider_model;

pub(crate) use channel::{create_channel_handler, delete_channel_handler, list_channels_handler};
pub(crate) use credential::{
    create_credential_handler, delete_credential_handler, list_credentials_handler,
    list_official_provider_configs_handler, upsert_official_provider_config_handler,
};
pub(crate) use model::{
    create_model_handler, create_model_price_handler, delete_model_handler,
    delete_model_price_handler, list_model_prices_handler, list_models_handler,
};
pub(crate) use provider::{
    create_provider_handler, delete_provider_handler, list_providers_handler,
    list_tenant_provider_readiness_handler,
};
pub(crate) use provider_model::{
    create_channel_model_handler, create_provider_account_handler, create_provider_model_handler,
    delete_channel_model_handler, delete_provider_account_handler, delete_provider_model_handler,
    list_channel_models_handler, list_provider_accounts_handler, list_provider_models_handler,
};

fn catalog_write_error_status(error: &anyhow::Error) -> StatusCode {
    let message = error.to_string();
    if message.contains("required")
        || message.contains("not registered")
        || message.contains("not bound")
        || message.contains("execution_instance_id must reference")
        || message.contains("provider-account can be saved")
        || message.contains("must exist before pricing can be saved")
        || message.contains("unsupported default_plugin_family")
        || message.contains("cannot override")
        || message.contains("must match")
    {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
