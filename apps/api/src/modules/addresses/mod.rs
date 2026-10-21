mod addresses_module;
mod controller;
mod dto;
mod entity;
mod repository;
mod service;

pub(crate) use addresses_module::routes;
pub(crate) use service::AddressesService;
