use crate::view::Mountable;
use std::fmt::Debug;

//pub type Rndr = dom::Dom;

/* #[cfg(feature = "testing")]
/// A renderer based on a mock DOM.
pub mod mock_dom;
/// A DOM renderer optimized for element creation.
#[cfg(feature = "sledgehammer")]
pub mod sledgehammer; */

/// Implements the instructions necessary to render an interface on some platform.
///
/// By default, this is implemented for the Document Object Model (DOM) in a Web
/// browser, but implementing this trait for some other platform allows you to use
/// the library to render any tree-based UI.
pub trait Renderer: Send + Sized + Debug + 'static {
    /// The basic type of node in the view tree.
    type Node: Mountable<Self> + Clone + 'static;
    /// A visible element in the view tree.
    type Element: AsRef<Self::Node>
        + CastFrom<Self::Node>
        + Mountable<Self>
        + Clone
        + 'static;
    /// A text node in the view tree.
    type Text: AsRef<Self::Node>
        + CastFrom<Self::Node>
        + Mountable<Self>
        + Clone
        + 'static;
    /// A placeholder node, which can be inserted into the tree but does not
    /// appear (e.g., a comment node in the DOM).
    type Placeholder: AsRef<Self::Node>
        + CastFrom<Self::Node>
        + Mountable<Self>
        + Clone
        + 'static;

    /// Interns a string slice, if that is available on this platform and useful as an optimization.
    fn intern(text: &str) -> &str;

    /// Creates a new text node.
    fn create_text_node(text: &str) -> Self::Text;

    /// Creates a new placeholder node.
    fn create_placeholder() -> Self::Placeholder;

    /// Sets the text content of the node. If it's not a text node, this does nothing.
    fn set_text(node: &Self::Text, text: &str);

    /// Appends the new child to the parent, before the anchor node. If `anchor` is `None`,
    /// append to the end of the parent's children.
    fn insert_node(
        parent: &Self::Element,
        new_child: &Self::Node,
        marker: Option<&Self::Node>,
    );

    /// Mounts the new child before the marker as its sibling.
    ///
    /// ## Panics
    /// The default implementation panics if `before` does not have a parent [`R::Element`].
    fn mount_before<M>(new_child: &mut M, before: &Self::Node)
    where
        M: Mountable<Self>,
    {
        let parent = Self::Element::cast_from(
            Self::get_parent(before).expect("could not find parent element"),
        )
        .expect("placeholder parent should be Element");
        new_child.mount(&parent, Some(before));
    }

    /// Tries to mount the new child before the marker as its sibling.
    ///
    /// Returns `false` if the child did not have a valid parent.
    #[track_caller]
    fn try_mount_before<M>(new_child: &mut M, before: &Self::Node) -> bool
    where
        M: Mountable<Self>,
    {
        if let Some(parent) =
            Self::get_parent(before).and_then(Self::Element::cast_from)
        {
            new_child.mount(&parent, Some(before));
            true
        } else {
            false
        }
    }

    /// Removes the child node from the parents, and returns the removed node.
    fn remove_node(
        parent: &Self::Element,
        child: &Self::Node,
    ) -> Option<Self::Node>;

    /// Removes all children from the parent element.
    fn clear_children(parent: &Self::Element);

    /// Removes the node.
    fn remove(node: &Self::Node);

    /// Gets the parent of the given node, if any.
    fn get_parent(node: &Self::Node) -> Option<Self::Node>;

    /// Returns the first child node of the given node, if any.
    fn first_child(node: &Self::Node) -> Option<Self::Node>;

    /// Returns the next sibling of the given node, if any.
    fn next_sibling(node: &Self::Node) -> Option<Self::Node>;

    /// Logs the given node in a platform-appropriate way.
    fn log_node(node: &Self::Node);
}

/// Attempts to cast from one type to another.
///
/// This works in a similar way to `TryFrom`. We implement it as a separate trait
/// simply so we don't have to create wrappers for the `web_sys` types; it can't be
/// implemented on them directly because of the orphan rules.
pub trait CastFrom<T>
where
    Self: Sized,
{
    /// Casts a node from one type to another.
    fn cast_from(source: T) -> Option<Self>;
}
