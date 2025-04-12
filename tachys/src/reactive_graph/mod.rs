use crate::{
    renderer::Renderer,
    view::{Mountable, Render},
};
use reactive_graph::effect::RenderEffect;
use std::sync::{Arc, Mutex};

mod owned;
mod suspense;

pub use owned::*;
pub use suspense::*;

impl<F, V, R> Render<R> for F
where
    F: ReactiveFunction<Output = V>,
    V: Render<R>,
    V::State: 'static,
    R: Renderer,
{
    type State = RenderEffectState<V::State>;

    #[track_caller]
    fn build(mut self) -> Self::State {
        let hook = throw_error::get_error_hook();
        RenderEffect::new(move |prev| {
            let _guard = hook
                .as_ref()
                .map(|h| throw_error::set_error_hook(Arc::clone(h)));
            let value = self.invoke();
            if let Some(mut state) = prev {
                value.rebuild(&mut state);
                state
            } else {
                value.build()
            }
        })
        .into()
    }

    #[track_caller]
    fn rebuild(self, state: &mut Self::State) {
        let new = self.build();
        let mut old = std::mem::replace(state, new);
        old.insert_before_this(state);
        old.unmount();
    }
}

/// Retained view state for a [`RenderEffect`].
pub struct RenderEffectState<T: 'static>(Option<RenderEffect<T>>);

impl<T> From<RenderEffect<T>> for RenderEffectState<T> {
    fn from(value: RenderEffect<T>) -> Self {
        Self(Some(value))
    }
}

impl<T, R> Mountable<R> for RenderEffectState<T>
where
    T: Mountable<R>,
    R: Renderer,
{
    fn unmount(&mut self) {
        if let Some(ref mut inner) = self.0 {
            inner.unmount();
        }
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        if let Some(ref mut inner) = self.0 {
            inner.mount(parent, marker);
        }
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        if let Some(inner) = &self.0 {
            inner.insert_before_this(child)
        } else {
            false
        }
    }
}

impl<M, R> Mountable<R> for RenderEffect<M>
where
    M: Mountable<R> + 'static,
    R: Renderer,
{
    fn unmount(&mut self) {
        self.with_value_mut(|state| state.unmount());
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        self.with_value_mut(|state| {
            state.mount(parent, marker);
        });
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        self.with_value_mut(|value| value.insert_before_this(child))
            .unwrap_or(false)
    }
}

impl<M, E, R> Mountable<R> for Result<M, E>
where
    M: Mountable<R>,
    R: Renderer,
{
    fn unmount(&mut self) {
        if let Ok(ref mut inner) = self {
            inner.unmount();
        }
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        if let Ok(ref mut inner) = self {
            inner.mount(parent, marker);
        }
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        if let Ok(inner) = &self {
            inner.insert_before_this(child)
        } else {
            false
        }
    }
}
/// A reactive function that can be shared across multiple locations and across threads.
pub type SharedReactiveFunction<T> = Arc<Mutex<dyn FnMut() -> T + Send>>;

/// A reactive view function.
pub trait ReactiveFunction: Send + 'static {
    /// The return type of the function.
    type Output;

    /// Call the function.
    fn invoke(&mut self) -> Self::Output;

    /// Converts the function into a cloneable, shared type.
    fn into_shared(self) -> Arc<Mutex<dyn FnMut() -> Self::Output + Send>>;
}

impl<T: 'static> ReactiveFunction for Arc<Mutex<dyn FnMut() -> T + Send>> {
    type Output = T;

    fn invoke(&mut self) -> Self::Output {
        let mut fun = self.lock().expect("lock poisoned");
        fun()
    }

    fn into_shared(self) -> Arc<Mutex<dyn FnMut() -> Self::Output + Send>> {
        self
    }
}

impl<T: Send + Sync + 'static> ReactiveFunction
    for Arc<dyn Fn() -> T + Send + Sync>
{
    type Output = T;

    fn invoke(&mut self) -> Self::Output {
        self()
    }

    fn into_shared(self) -> Arc<Mutex<dyn FnMut() -> Self::Output + Send>> {
        Arc::new(Mutex::new(move || self()))
    }
}

impl<F, T> ReactiveFunction for F
where
    F: FnMut() -> T + Send + 'static,
{
    type Output = T;

    fn invoke(&mut self) -> Self::Output {
        self()
    }

    fn into_shared(self) -> Arc<Mutex<dyn FnMut() -> Self::Output + Send>> {
        Arc::new(Mutex::new(self))
    }
}

macro_rules! reactive_impl {
    ($name:ident, <$($gen:ident),*>, $v:ty, $( $where_clause:tt )*) =>
    {
        #[allow(deprecated)]
        impl<R,$($gen),*> Render<R> for $name<$($gen),*>
        where
            $v: Render<R> + Clone + Send + Sync + 'static,
            <$v as Render<R>>::State: 'static,
            R: Renderer,
            $($where_clause)*

        {
            type State = RenderEffectState<<$v as Render<R>>::State>;

            #[track_caller]
            fn build(self) -> Self::State {
                (move || self.get()).build()
            }

            #[track_caller]
            fn rebuild(self, state: &mut Self::State) {
                let new = self.build();
                let mut old = std::mem::replace(state, new);
                old.insert_before_this(state);
                old.unmount();
            }
        }
    };
}

#[cfg(not(feature = "nightly"))]
mod stable {
    use super::RenderEffectState;
    use crate::{
        renderer::Renderer,
        view::{Mountable, Render},
    };
    #[allow(deprecated)]
    use reactive_graph::wrappers::read::MaybeSignal;
    use reactive_graph::{
        computed::{ArcMemo, Memo},
        owner::Storage,
        signal::{ArcReadSignal, ArcRwSignal, ReadSignal, RwSignal},
        traits::Get,
        wrappers::read::{ArcSignal, Signal},
    };

    reactive_impl!(
        RwSignal,
        <V, S>,
        V,
        RwSignal<V, S>: Get<Value = V>,
        S: Storage<V> + Storage<Option<V>>,
        S: Send + Sync + 'static,
    );
    reactive_impl!(
        ReadSignal,
        <V, S>,
        V,
        ReadSignal<V, S>: Get<Value = V>,
        S: Storage<V> + Storage<Option<V>>,
        S: Send + Sync + 'static,
    );
    reactive_impl!(
        Memo,
        <V, S>,
        V,
        Memo<V, S>: Get<Value = V>,
        S: Storage<V> + Storage<Option<V>>,
        S: Send + Sync + 'static,
    );
    reactive_impl!(
        Signal,
        <V, S>,
        V,
        Signal<V, S>: Get<Value = V>,
        S: Storage<V> + Storage<Option<V>>,
        S: Send + Sync + 'static,
    );
    reactive_impl!(
        MaybeSignal,
        <V, S>,
        V,
        MaybeSignal<V, S>: Get<Value = V>,
        S: Storage<V> + Storage<Option<V>>,
        S: Send + Sync + 'static,
    );
    reactive_impl!(ArcRwSignal, <V>, V, ArcRwSignal<V>: Get<Value = V>);
    reactive_impl!(ArcReadSignal, <V>, V, ArcReadSignal<V>: Get<Value = V>);
    reactive_impl!(ArcMemo, <V>, V, ArcMemo<V>: Get<Value = V>);
    reactive_impl!(ArcSignal, <V>, V, ArcSignal<V>: Get<Value = V>);
}

#[cfg(feature = "reactive_stores")]
mod reactive_stores {
    use super::{RenderEffectState, Renderer};
    use crate::view::{Mountable, Render};
    #[allow(deprecated)]
    use reactive_graph::{owner::Storage, traits::Get};
    use reactive_stores::{
        ArcField, ArcStore, AtIndex, AtKeyed, DerefedField, Field,
        KeyedSubfield, Store, StoreField, Subfield,
    };
    use std::ops::{Deref, DerefMut, Index, IndexMut};

    reactive_impl!(
        Subfield,
        <Inner, Prev, V>,
        V,
        Subfield<Inner, Prev, V>: Get<Value = V>,
        Prev: Send + Sync + 'static,
        Inner: Send + Sync + Clone + 'static,
    );

    reactive_impl!(
        AtKeyed,
        <Inner, Prev, K, V>,
        V,
        AtKeyed<Inner, Prev, K, V>: Get<Value = V>,
        Prev: Send + Sync + 'static,
        Inner: Send + Sync + Clone + 'static,
        K: Send + Sync + std::fmt::Debug + Clone + 'static,
        for<'a> &'a V: IntoIterator,
    );

    reactive_impl!(
        KeyedSubfield,
        <Inner, Prev, K, V>,
        V,
        KeyedSubfield<Inner, Prev, K, V>: Get<Value = V>,
        Prev: Send + Sync + 'static,
        Inner: Send + Sync + Clone + 'static,
        K: Send + Sync + std::fmt::Debug + Clone + 'static,
        for<'a> &'a V: IntoIterator,
    );

    reactive_impl!(
        DerefedField,
        <S>,
        <S::Value as Deref>::Target,
        S: Clone + StoreField + Send + Sync + 'static,
        <S as StoreField>::Value: Deref + DerefMut
    );

    reactive_impl!(
        AtIndex,
        <Inner, Prev>,
        <Prev as Index<usize>>::Output,
        AtIndex<Inner, Prev>: Get<Value = Prev::Output>,
        Prev: Send + Sync + IndexMut<usize> + 'static,
        Inner: Send + Sync + Clone + 'static,
    );
    reactive_impl!(
        Store,
        <V, S>,
        V,
        Store<V, S>: Get<Value = V>,
        S: Storage<V> + Storage<Option<V>>,
        S: Send + Sync + 'static,
    );
    reactive_impl!(
        Field,
        <V, S>,
        V,
        Field<V, S>: Get<Value = V>,
        S: Storage<V> + Storage<Option<V>>,
        S: Send + Sync + 'static,
    );
    reactive_impl!(ArcStore, <V>, V, ArcStore<V>: Get<Value = V>);
    reactive_impl!(ArcField, <V>, V, ArcField<V>: Get<Value = V>);
}

/*
#[cfg(test)]
mod tests {
    use crate::{
        html::element::{button, main, HtmlElement},
        renderer::mock_dom::MockDom,
        view::Render,
    };
    use leptos_reactive::{create_runtime, RwSignal, SignalGet, SignalSet};

    #[test]
    fn create_dynamic_element() {
        let rt = create_runtime();
        let count = RwSignal::new(0);
        let app: HtmlElement<_, _, _, MockDom> =
            button((), move || count.get().to_string());
        let el = app.build();
        assert_eq!(el.el.to_debug_html(), "<button>0</button>");
        rt.dispose();
    }

    #[test]
    fn update_dynamic_element() {
        let rt = create_runtime();
        let count = RwSignal::new(0);
        let app: HtmlElement<_, _, _, MockDom> =
            button((), move || count.get().to_string());
        let el = app.build();
        assert_eq!(el.el.to_debug_html(), "<button>0</button>");
        count.set(1);
        assert_eq!(el.el.to_debug_html(), "<button>1</button>");
        rt.dispose();
    }

    #[test]
    fn update_dynamic_element_among_siblings() {
        let rt = create_runtime();
        let count = RwSignal::new(0);
        let app: HtmlElement<_, _, _, MockDom> = main(
            (),
            button(
                (),
                ("Hello, my ", move || count.get().to_string(), " friends."),
            ),
        );
        let el = app.build();
        assert_eq!(
            el.el.to_debug_html(),
            "<main><button>Hello, my 0 friends.</button></main>"
        );
        count.set(42);
        assert_eq!(
            el.el.to_debug_html(),
            "<main><button>Hello, my 42 friends.</button></main>"
        );
        rt.dispose();
    }
}
 */
