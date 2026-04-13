use super::*;

mod auth;
mod billing;
mod catalog;
mod commerce;
mod gateway;
mod marketing;
mod pricing;
mod routing;
mod runtime;
mod tenant;
mod user;

pub(crate) use auth::*;
pub(crate) use billing::*;
pub(crate) use catalog::*;
pub(crate) use commerce::*;
pub(crate) use gateway::*;
pub(crate) use marketing::*;
pub(crate) use pricing::*;
pub(crate) use routing::*;
pub(crate) use runtime::*;
pub(crate) use tenant::*;
pub(crate) use user::*;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ErrorResponse {
    pub(crate) error: ErrorBody,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ErrorBody {
    pub(crate) message: String,
}

fn default_true() -> bool {
    true
}

fn default_window_seconds() -> u64 {
    60
}
