mod assembly;
mod entries;

pub use entries::{
    load_marketing_coupon_context_by_value, load_marketing_coupon_context_for_code_id,
    load_marketing_coupon_context_for_reference, load_marketing_coupon_context_from_code_record,
};
