mod controller;
mod dto;
mod entity;
mod orders_module;
mod repository;
mod service;

pub(crate) use orders_module::routes;
pub(crate) use service::OrdersService;
