//! Allows rendering user interfaces based on a statically-typed view tree.
//!
//! This view tree is generic over rendering backends, and agnostic about reactivity/change
//! detection.

// this is specifically used for `unsized_const_params` below
// this allows us to use const generic &'static str for static text nodes and attributes
#![allow(incomplete_features)]
#![cfg_attr(
    all(feature = "nightly", rustc_nightly),
    feature(unsized_const_params)
)]
// support for const generic &'static str has now moved back and forth between
// these two features a couple times; we'll just enable both
#![cfg_attr(all(feature = "nightly", rustc_nightly), feature(adt_const_params))]
#![deny(missing_docs)]

/// Commonly-used traits.
pub mod prelude {
    pub use crate::{
        renderer::Renderer,
        view::{
            any_view::{AnyView, IntoAny, IntoMaybeErased},
            IntoRender, Mountable, Render,
        },
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

/// A type-erased container.
pub mod erased;
