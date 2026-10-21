mod controller;
mod dto;
mod entity;
mod repository;
mod restaurant_settings_module;
mod service;

pub(crate) use restaurant_settings_module::routes;
pub(crate) use service::RestaurantSettingsService;
