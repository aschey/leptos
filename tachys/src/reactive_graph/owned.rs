use crate::{
    prelude::{Mountable, Renderer},
    view::Render,
};
use reactive_graph::owner::Owner;
use std::marker::PhantomData;

/// A view wrapper that sets the reactive [`Owner`] to a particular owner whenever it is rendered.
#[derive(Debug, Clone)]
pub struct OwnedView<T> {
    owner: Owner,
    view: T,
}

impl<T> OwnedView<T> {
    /// Wraps a view with the current owner.
    pub fn new(view: T) -> Self {
        let owner = Owner::current().expect("no reactive owner");
        Self { owner, view }
    }

    /// Wraps a view with the given owner.
    pub fn new_with_owner(view: T, owner: Owner) -> Self {
        Self { owner, view }
    }
}

/// Retained view state for an [`OwnedView`].
#[derive(Debug, Clone)]
pub struct OwnedViewState<T, R>
where
    T: Mountable<R>,
    R: Renderer,
{
    owner: Owner,
    state: T,
    rndr: PhantomData<R>,
}

impl<T, R> OwnedViewState<T, R>
where
    T: Mountable<R>,
    R: Renderer,
{
    /// Wraps a state with the given owner.
    fn new(state: T, owner: Owner) -> Self {
        Self {
            owner,
            state,
            rndr: Default::default(),
        }
    }
}

impl<T, R> Render<R> for OwnedView<T>
where
    T: Render<R>,
    R: Renderer,
{
    type State = OwnedViewState<T::State, R>;

    fn build(self) -> Self::State {
        let state = self.owner.with(|| self.view.build());
        OwnedViewState::new(state, self.owner)
    }

    fn rebuild(self, state: &mut Self::State) {
        let OwnedView { owner, view, .. } = self;
        owner.with(|| view.rebuild(&mut state.state));
        state.owner = owner;
    }
}

impl<T, R> Mountable<R> for OwnedViewState<T, R>
where
    T: Mountable<R>,
    R: Renderer,
{
    fn unmount(&mut self) {
        self.state.unmount();
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        self.state.mount(parent, marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        self.state.insert_before_this(child)
    }
}
