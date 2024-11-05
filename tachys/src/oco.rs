use crate::prelude::{Mountable, Render, Renderer};
use oco_ref::Oco;

/// Retained view state for [`Oco`].
pub struct OcoStrState<R>
where
    R: Renderer,
{
    node: R::Text,
    str: Oco<'static, str>,
}

impl<R> Render<R> for Oco<'static, str>
where
    R: Renderer,
{
    type State = OcoStrState<R>;

    fn build(self) -> Self::State {
        let node = R::create_text_node(&self);
        OcoStrState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let OcoStrState { node, str } = state;
        if &self != str {
            R::set_text(node, &self);
            *str = self;
        }
    }
}

impl<R> Mountable<R> for OcoStrState<R>
where
    R: Renderer,
{
    fn unmount(&mut self) {
        self.node.unmount()
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        R::insert_node(parent, self.node.as_ref(), marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        self.node.insert_before_this(child)
    }
}
