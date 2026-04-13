use super::*;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct PublishCommercialCatalogPublicationRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ScheduleCommercialCatalogPublicationRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RetireCommercialCatalogPublicationRequest {
    pub(crate) reason: String,
}
