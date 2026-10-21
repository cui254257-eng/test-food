use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct Address {
    pub(crate) id: i64,
    pub(crate) recipient_name: String,
    pub(crate) phone: String,
    pub(crate) province: String,
    pub(crate) city: String,
    pub(crate) district: String,
    pub(crate) detail: String,
    pub(crate) is_default: bool,
}

#[derive(Debug, FromRow)]
pub(crate) struct AddressDefaultState {
    pub(crate) is_default: bool,
}
