mod controller;
mod dto;
mod entity;
mod products_module;
mod repository;
mod service;

pub(crate) use dto::ProductSpecificationGroup;
pub(crate) use products_module::routes;
pub(crate) use service::ProductsService;
