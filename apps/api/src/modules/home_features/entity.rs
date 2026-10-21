use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct HomeFeature {
    pub(crate) id: i64,
    pub(crate) label: String,
    pub(crate) icon: String,
    pub(crate) target_path: String,
    pub(crate) sort_order: i32,
    pub(crate) is_active: bool,
}
