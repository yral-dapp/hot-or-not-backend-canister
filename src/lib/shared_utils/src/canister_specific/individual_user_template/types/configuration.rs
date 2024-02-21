use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct IndividualUserConfiguration {
    pub url_to_send_canister_metrics_to: Option<String>,
}
