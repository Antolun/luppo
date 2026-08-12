#![allow(non_local_definitions)]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod actionsapi;
pub mod build;
pub mod components;
pub mod flags;
pub mod package;
pub mod python_api;
pub mod reset_history;
pub mod sandbox;

rust_i18n::i18n!("../locales", fallback = "tr");

// İleride buraya build_package, cleanup gibi ana işlevler eklenecektir.
