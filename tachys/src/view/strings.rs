use super::{Mountable, Render};
use crate::prelude::Renderer;
use std::{borrow::Cow, rc::Rc, sync::Arc};

/// Retained view state for `&str`.
pub struct StrState<'a, R>
where
    R: Renderer,
{
    pub(crate) node: R::Text,
    str: &'a str,
}

impl<'a, R> Render<R> for &'a str
where
    R: Renderer,
{
    type State = StrState<'a, R>;

    fn build(self) -> Self::State {
        let node = R::create_text_node(self);
        StrState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let StrState { node, str } = state;
        if &self != str {
            R::set_text(node, self);
            *str = self;
        }
    }
}

impl<'a, R> Mountable<R> for StrState<'a, R>
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

/// Retained view state for `String`.
pub struct StringState<R>
where
    R: Renderer,
{
    node: R::Text,
    str: String,
}

impl<R> Render<R> for String
where
    R: Renderer,
{
    type State = StringState<R>;

    fn build(self) -> Self::State {
        let node = R::create_text_node(&self);
        StringState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let StringState { node, str } = state;
        if &self != str {
            R::set_text(node, &self);
            *str = self;
        }
    }
}

impl<R> Mountable<R> for StringState<R>
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

/// Retained view state for `Rc<str>`.
pub struct RcStrState<R>
where
    R: Renderer,
{
    node: R::Text,
    str: Rc<str>,
}

impl<R> Render<R> for Rc<str>
where
    R: Renderer,
{
    type State = RcStrState<R>;

    fn build(self) -> Self::State {
        let node = R::create_text_node(&self);
        RcStrState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let RcStrState { node, str } = state;
        if !Rc::ptr_eq(&self, str) {
            R::set_text(node, &self);
            *str = self;
        }
    }
}

impl<R> Mountable<R> for RcStrState<R>
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

/// Retained view state for `Arc<str>`.
pub struct ArcStrState<R>
where
    R: Renderer,
{
    node: R::Text,
    str: Arc<str>,
}

impl<R> Render<R> for Arc<str>
where
    R: Renderer,
{
    type State = ArcStrState<R>;

    fn build(self) -> Self::State {
        let node = R::create_text_node(&self);
        ArcStrState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let ArcStrState { node, str } = state;
        if !Arc::ptr_eq(&self, str) {
            R::set_text(node, &self);
            *str = self;
        }
    }
}

impl<R> Mountable<R> for ArcStrState<R>
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

/// Retained view state for `Cow<'_, str>`.
pub struct CowStrState<'a, R>
where
    R: Renderer,
{
    node: R::Text,
    str: Cow<'a, str>,
}

impl<'a, R> Render<R> for Cow<'a, str>
where
    R: Renderer,
{
    type State = CowStrState<'a, R>;

    fn build(self) -> Self::State {
        let node = R::create_text_node(&self);
        CowStrState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let CowStrState { node, str } = state;
        if self != *str {
            R::set_text(node, &self);
            *str = self;
        }
    }
}

impl<'a, R> Mountable<R> for CowStrState<'a, R>
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
