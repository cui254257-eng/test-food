use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct AddressInput {
    pub(crate) recipient_name: String,
    pub(crate) phone: String,
    pub(crate) province: String,
    pub(crate) city: String,
    pub(crate) district: String,
    pub(crate) detail: String,
    #[serde(default)]
    pub(crate) is_default: bool,
}
