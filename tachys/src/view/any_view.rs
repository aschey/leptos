use super::{Mountable, Render};
use crate::{
    erased::{Erased, ErasedLocal},
    prelude::Renderer,
};
use std::{any::TypeId, fmt::Debug};

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
    value: Erased,
    build: fn(Erased) -> AnyViewState<R>,
    rebuild: fn(Erased, &mut AnyViewState<R>),
}

impl<R> Debug for AnyView<R>
where
    R: Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyView")
            .field("type_id", &self.type_id)
            .finish_non_exhaustive()
    }
}
/// Retained view state for [`AnyView`].
pub struct AnyViewState<R>
where
    R: Renderer,
{
    type_id: TypeId,
    state: ErasedLocal,
    unmount: fn(&mut ErasedLocal),
    mount: fn(&mut ErasedLocal, parent: &R::Element, marker: Option<&R::Node>),
    insert_before_this: fn(&ErasedLocal, child: &mut dyn Mountable<R>) -> bool,
}

impl<R> Debug for AnyViewState<R>
where
    R: Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyViewState")
            .field("type_id", &self.type_id)
            .field("state", &"")
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

/// A more general version of [`IntoAny`] that allows into [`AnyView`],
/// but also erasing other types that don't implement [`RenderHtml`] like routing.
pub trait IntoMaybeErased<R> {
    /// The type of the output.
    type Output: IntoMaybeErased<R>;

    /// Converts the view into a type-erased view if in erased mode.
    fn into_maybe_erased(self) -> Self::Output;
}

impl<T, R> IntoMaybeErased<R> for T
where
    T: Render<R>,
    T: IntoAny<R>,
    R: Renderer,
{
    #[cfg(not(erase_components))]
    type Output = Self;

    #[cfg(erase_components)]
    type Output = AnyView<R>;

    fn into_maybe_erased(self) -> Self::Output {
        #[cfg(not(erase_components))]
        {
            self
        }
        #[cfg(erase_components)]
        {
            self.into_any()
        }
    }
}

fn mount_any<T, R>(
    state: &mut ErasedLocal,
    parent: &R::Element,
    marker: Option<&R::Node>,
) where
    T: Render<R>,
    T::State: 'static,
    R: Renderer,
{
    state.get_mut::<T::State>().mount(parent, marker)
}

fn unmount_any<T, R>(state: &mut ErasedLocal)
where
    T: Render<R>,
    T::State: 'static,
    R: Renderer,
{
    state.get_mut::<T::State>().unmount();
}

fn insert_before_this<T, R>(
    state: &ErasedLocal,
    child: &mut dyn Mountable<R>,
) -> bool
where
    T: Render<R>,
    T::State: 'static,
    R: Renderer,
{
    state.get_ref::<T::State>().insert_before_this(child)
}

impl<T, R> IntoAny<R> for T
where
    T: Send,
    T: Render<R> + 'static,
    T::State: 'static,
    R: Renderer,
{
    fn into_any(self) -> AnyView<R> {
        fn build<T: Render<R> + 'static, R: Renderer>(
            value: Erased,
        ) -> AnyViewState<R> {
            let state = ErasedLocal::new(value.into_inner::<T>().build());
            AnyViewState {
                type_id: TypeId::of::<T>(),
                state,
                mount: mount_any::<T, R>,
                unmount: unmount_any::<T, R>,
                insert_before_this: insert_before_this::<T, R>,
            }
        }

        fn rebuild<T: Render<R> + 'static, R: Renderer>(
            value: Erased,
            state: &mut AnyViewState<R>,
        ) {
            let state = state.state.get_mut::<<T as Render<R>>::State>();
            value.into_inner::<T>().rebuild(state);
        }

        AnyView {
            type_id: TypeId::of::<T>(),
            value: Erased::new(self),
            build: build::<T, R>,
            rebuild: rebuild::<T, R>,
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
        if self.type_id == state.type_id {
            (self.rebuild)(self.value, state)
        } else {
            let mut new = self.build();
            state.insert_before_this(&mut new);
            state.unmount();
            *state = new;
        }
    }
}

impl<R> Mountable<R> for AnyViewState<R>
where
    R: Renderer,
{
    fn unmount(&mut self) {
        (self.unmount)(&mut self.state)
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        (self.mount)(&mut self.state, parent, marker)
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        (self.insert_before_this)(&self.state, child)
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
