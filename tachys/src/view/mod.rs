use crate::renderer::Renderer;
use parking_lot::RwLock;
use std::{cell::RefCell, rc::Rc, sync::Arc};

pub mod any_view;
/// Allows choosing between one of several views.
pub mod either;
/// View rendering for `Result<_, _>` types.
pub mod error_boundary;
/// A type-erased view collection.
pub mod fragment;
/// View implementations for several iterable types.
pub mod iterators;
/// Keyed list iteration.
pub mod keyed;
mod primitives;
/// Optimized types for static strings known at compile time.
#[cfg(feature = "nightly")]
pub mod static_types;
/// View implementation for string types.
pub mod strings;
/// View implementations for tuples.
pub mod tuples;

/// The `Render` trait allows rendering something as part of the user interface.
///
/// It is generic over the renderer itself, as long as that implements the [`Renderer`]
/// trait.
pub trait Render<R: Renderer>: Sized {
    /// The “view state” for this type, which can be retained between updates.
    ///
    /// For example, for a text node, `State` might be the actual DOM text node
    /// and the previous string, to allow for diffing between updates.
    type State: Mountable<R>;

    /// Creates the view for the first time, without hydrating from existing HTML.
    fn build(self) -> Self::State;

    /// Updates the view with new data.
    fn rebuild(self, state: &mut Self::State);
}

/// Allows a type to be mounted to the DOM.
pub trait Mountable<R: Renderer> {
    /// Detaches the view from the DOM.
    fn unmount(&mut self);

    /// Mounts a node to the interface.
    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>);

    /// Inserts another `Mountable` type before this one. Returns `false` if
    /// this does not actually exist in the UI (for example, `()`).
    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool;

    /// Inserts another `Mountable` type before this one, or before the marker
    /// if this one doesn't exist in the UI (for example, `()`).
    fn insert_before_this_or_marker(
        &self,
        parent: &R::Element,
        child: &mut dyn Mountable<R>,
        marker: Option<&R::Node>,
    ) {
        if !self.insert_before_this(child) {
            child.mount(parent, marker);
        }
    }
}

/// Indicates where a node should be mounted to its parent.
pub enum MountKind<R>
where
    R: Renderer,
{
    /// Node should be mounted before this marker node.
    Before(R::Node),
    /// Node should be appended to the parent’s children.
    Append,
}

impl<T, R> Mountable<R> for Option<T>
where
    T: Mountable<R>,
    R: Renderer,
{
    fn unmount(&mut self) {
        if let Some(ref mut mounted) = self {
            mounted.unmount()
        }
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        if let Some(ref mut inner) = self {
            inner.mount(parent, marker);
        }
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        self.as_ref()
            .map(|inner| inner.insert_before_this(child))
            .unwrap_or(false)
    }
}

impl<T, R> Mountable<R> for Rc<RefCell<T>>
where
    T: Mountable<R>,
    R: Renderer,
{
    fn unmount(&mut self) {
        self.borrow_mut().unmount()
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        self.borrow_mut().mount(parent, marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        self.borrow().insert_before_this(child)
    }
}

/// Keeps track of what position the item currently being hydrated is in, relative to its siblings
/// and parents.
#[derive(Debug, Default, Clone)]
pub struct PositionState(Arc<RwLock<Position>>);

impl PositionState {
    /// Creates a new position tracker.
    pub fn new(position: Position) -> Self {
        Self(Arc::new(RwLock::new(position)))
    }

    /// Sets the current position.
    pub fn set(&self, position: Position) {
        *self.0.write() = position;
    }

    /// Gets the current position.
    pub fn get(&self) -> Position {
        *self.0.read()
    }

    /// Creates a new [`PositionState`], which starts with the same [`Position`], but no longer
    /// shares data with this `PositionState`.
    pub fn deep_clone(&self) -> Self {
        let current = self.get();
        Self(Arc::new(RwLock::new(current)))
    }
}

/// The position of this element, relative to others.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Position {
    /// This is the current node.
    Current,
    /// This is the first child of its parent.
    #[default]
    FirstChild,
    /// This is the next child after another child.
    NextChild,
    /// This is the next child after a text node.
    NextChildAfterText,
    /// This is the only child of its parent.
    OnlyChild,
    /// This is the last child of its parent.
    LastChild,
}
