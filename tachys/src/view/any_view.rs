use super::{Mountable, Render};
use crate::prelude::Renderer;
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

/// A type-erased view. This can be used if control flow requires that multiple different types of
/// view must be received, and it is either impossible or too cumbersome to use the `EitherOf___`
/// enums.
///
/// It can also be used to create recursive components, which otherwise cannot return themselves
/// due to the static typing of the view tree.
///
/// Generally speaking, using `AnyView` restricts the amount of information available to the
/// compiler and should be limited to situations in which it is necessary to preserve the maximum
/// amount of type information possible.
pub struct AnyView<R>
where
    R: Renderer,
{
    type_id: TypeId,
    value: Box<dyn Any + Send>,
    build: fn(Box<dyn Any>) -> AnyViewState<R>,
    rebuild: fn(TypeId, Box<dyn Any>, &mut AnyViewState<R>),
}

/// Retained view state for [`AnyView`].
pub struct AnyViewState<R>
where
    R: Renderer,
{
    type_id: TypeId,
    state: Box<dyn Any>,
    unmount: fn(&mut dyn Any),
    mount: fn(&mut dyn Any, parent: &R::Element, marker: Option<&R::Node>),
    insert_before_this: fn(&dyn Any, child: &mut dyn Mountable<R>) -> bool,
}

impl<R> Debug for AnyViewState<R>
where
    R: Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyViewState")
            .field("type_id", &self.type_id)
            .field("state", &self.state)
            .field("unmount", &self.unmount)
            .field("mount", &self.mount)
            .field("insert_before_this", &self.insert_before_this)
            .finish()
    }
}

/// Allows converting some view into [`AnyView`].
pub trait IntoAny<R>
where
    R: Renderer,
{
    /// Converts the view into a type-erased [`AnyView`].
    fn into_any(self) -> AnyView<R>;
}

fn mount_any<T, R>(
    state: &mut dyn Any,
    parent: &R::Element,
    marker: Option<&R::Node>,
) where
    T: Render<R>,
    T::State: 'static,
    R: Renderer,
{
    let state = state
        .downcast_mut::<T::State>()
        .expect("AnyViewState::as_mountable couldn't downcast state");
    state.mount(parent, marker)
}

fn unmount_any<T, R>(state: &mut dyn Any)
where
    T: Render<R>,
    T::State: 'static,
    R: Renderer,
{
    let state = state
        .downcast_mut::<T::State>()
        .expect("AnyViewState::unmount couldn't downcast state");
    state.unmount();
}

fn insert_before_this<T, R>(
    state: &dyn Any,
    child: &mut dyn Mountable<R>,
) -> bool
where
    T: Render<R>,
    T::State: 'static,
    R: Renderer,
{
    let state = state
        .downcast_ref::<T::State>()
        .expect("AnyViewState::insert_before_this couldn't downcast state");
    state.insert_before_this(child)
}

impl<T, R> IntoAny<R> for T
where
    T: Send,
    T: Render<R> + 'static,
    T::State: 'static,
    R: Renderer,
{
    // inlining allows the compiler to remove the unused functions
    // i.e., doesn't ship HTML-generating code that isn't used
    #[inline(always)]
    fn into_any(self) -> AnyView<R> {
        let value = Box::new(self) as Box<dyn Any + Send>;
        let build = |value: Box<dyn Any>| {
            let value = value
                .downcast::<T>()
                .expect("AnyView::build couldn't downcast");
            let state = Box::new(value.build());

            AnyViewState {
                type_id: TypeId::of::<T>(),
                state,

                mount: mount_any::<T, R>,
                unmount: unmount_any::<T, R>,
                insert_before_this: insert_before_this::<T, R>,
            }
        };

        let rebuild = |new_type_id: TypeId,
                       value: Box<dyn Any>,
                       state: &mut AnyViewState<R>| {
            let value = value
                .downcast::<T>()
                .expect("AnyView::rebuild couldn't downcast value");
            if new_type_id == state.type_id {
                let state = state
                    .state
                    .downcast_mut()
                    .expect("AnyView::rebuild couldn't downcast state");
                value.rebuild(state);
            } else {
                let mut new = value.into_any().build();
                state.insert_before_this(&mut new);
                state.unmount();
                *state = new;
            }
        };

        AnyView {
            type_id: TypeId::of::<T>(),
            value,
            build,
            rebuild,
        }
    }
}

impl<R> Render<R> for AnyView<R>
where
    R: Renderer,
{
    type State = AnyViewState<R>;

    fn build(self) -> Self::State {
        (self.build)(self.value)
    }

    fn rebuild(self, state: &mut Self::State) {
        (self.rebuild)(self.type_id, self.value, state)
    }
}

impl<R> Mountable<R> for AnyViewState<R>
where
    R: Renderer,
{
    fn unmount(&mut self) {
        (self.unmount)(&mut *self.state)
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        (self.mount)(&mut *self.state, parent, marker)
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        (self.insert_before_this)(&*self.state, child)
    }
}
/*
#[cfg(test)]
mod tests {
    use super::IntoAny;
    use crate::{
        html::element::{p, span},
        renderer::mock_dom::MockDom,
        view::{any_view::AnyView, RenderHtml},
    };

    #[test]
    fn should_handle_html_creation() {
        let x = 1;
        let mut buf = String::new();
        let view: AnyView<MockDom> = if x == 0 {
            p((), "foo").into_any()
        } else {
            span((), "bar").into_any()
        };
        view.to_html(&mut buf, &Default::default());
        assert_eq!(buf, "<span>bar</span><!>");
    }
}
 */
