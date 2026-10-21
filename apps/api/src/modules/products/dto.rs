use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct ProductSpecificationGroup {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) required: bool,
    pub(crate) options: Vec<ProductSpecificationOption>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct ProductSpecificationOption {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) price_delta: f64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProductInput {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) category: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) specifications: Vec<ProductSpecificationGroup>,
    pub(crate) price: f64,
    pub(crate) stock: i64,
    pub(crate) status: String,
    #[serde(default = "default_discount_eligible")]
    pub(crate) discount_eligible: bool,
    pub(crate) image_data: Option<String>,
}

fn default_discount_eligible() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProductQuery {
    pub(crate) q: Option<String>,
    pub(crate) status: Option<String>,
}
