//! Builds the VitePress config reference from every schema registered with
//! `wayle_config::register_module!`. Start at [`generator::DocsGenerator`];
//! it reads the [`registry`] and hands each entry to [`module_page`], then
//! emits the shared [`types_page`] and [`index_page`] alongside.

pub mod generator;
pub mod index_page;
pub mod module_page;
pub mod registry;
pub mod rustdoc;
pub mod types_page;

pub use generator::Error;
