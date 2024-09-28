//! Allows rendering user interfaces based on a statically-typed view tree.
//!
//! This view tree is generic over rendering backends, and agnostic about reactivity/change
//! detection.

#![allow(incomplete_features)] // yolo
#![cfg_attr(feature = "nightly", feature(unsized_const_params))]
//#![deny(missing_docs)]

/// Commonly-used traits.
pub mod prelude {
    pub use crate::{
        renderer::Renderer,
        view::{any_view::IntoAny, Mountable, Render},
    };
}

/// Defines various backends that can render views.
pub mod renderer;
/// Core logic for manipulating views.
pub mod view;

pub use either_of as either;

/// View implementations for the `oco_ref` crate (cheaply-cloned string types).
#[cfg(feature = "oco")]
pub mod oco;
/// View implementations for the `reactive_graph` crate.
#[cfg(feature = "reactive_graph")]
pub mod reactive_graph;
