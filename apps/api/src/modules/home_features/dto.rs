use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct HomeFeatureInput {
    pub(crate) label: String,
    pub(crate) icon: String,
    pub(crate) target_path: String,
    pub(crate) sort_order: i32,
    pub(crate) is_active: bool,
}
