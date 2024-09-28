use super::{Mountable, Render};
use crate::prelude::Renderer;

/// A static string that is known at compile time and can be optimized by including its type in the
/// view tree.
#[derive(Debug, Clone, Copy)]
pub struct Static<const V: &'static str>;

impl<const V: &'static str> PartialEq for Static<V> {
    fn eq(&self, _other: &Self) -> bool {
        // by definition, two static values of same const V are same
        true
    }
}

impl<const V: &'static str> AsRef<str> for Static<V> {
    fn as_ref(&self) -> &str {
        V
    }
}

impl<const V: &'static str, R> Render<R> for Static<V>
where
    R::Text: Mountable<R>,
    R: Renderer,
{
    type State = Option<R::Text>;

    fn build(self) -> Self::State {
        // a view state has to be returned so it can be mounted
        Some(R::create_text_node(V))
    }

    // This type is specified as static, so no rebuilding is done.
    fn rebuild(self, _state: &mut Self::State) {}
}
