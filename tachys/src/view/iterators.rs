use super::{Mountable, Render};
use crate::prelude::Renderer;
use either_of::Either;
use itertools::Itertools;
use std::marker::PhantomData;

/// Retained view state for an `Option`.
pub type OptionState<T, R> =
    Either<<T as Render<R>>::State, <() as Render<R>>::State>;

impl<T, R> Render<R> for Option<T>
where
    T: Render<R>,
    R: Renderer,
{
    type State = OptionState<T, R>;

    fn build(self) -> Self::State {
        match self {
            Some(value) => Either::Left(value),
            None => Either::Right(()),
        }
        .build()
    }

    fn rebuild(self, state: &mut Self::State) {
        match self {
            Some(value) => Either::Left(value),
            None => Either::Right(()),
        }
        .rebuild(state)
    }
}

impl<T, R> Render<R> for Vec<T>
where
    T: Render<R>,
    R: Renderer,
{
    type State = VecState<T::State, R>;

    fn build(self) -> Self::State {
        let marker = R::create_placeholder();
        VecState {
            states: self.into_iter().map(T::build).collect(),
            marker,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        let VecState { states, marker } = state;
        let old = states;
        // this is an unkeyed diff
        if old.is_empty() {
            let mut new = self.build().states;
            for item in new.iter_mut() {
                R::mount_before(item, marker.as_ref());
            }
            *old = new;
        } else if self.is_empty() {
            // TODO fast path for clearing
            for item in old.iter_mut() {
                item.unmount();
            }
            old.clear();
        } else {
            let mut adds = vec![];
            let mut removes_at_end = 0;
            for item in self.into_iter().zip_longest(old.iter_mut()) {
                match item {
                    itertools::EitherOrBoth::Both(new, old) => {
                        T::rebuild(new, old)
                    }
                    itertools::EitherOrBoth::Left(new) => {
                        let mut new_state = new.build();
                        R::mount_before(&mut new_state, marker.as_ref());
                        adds.push(new_state);
                    }
                    itertools::EitherOrBoth::Right(old) => {
                        removes_at_end += 1;
                        old.unmount()
                    }
                }
            }
            old.truncate(old.len() - removes_at_end);
            old.append(&mut adds);
        }
    }
}

/// Retained view state for a `Vec<_>`.
pub struct VecState<T, R>
where
    T: Mountable<R>,
    R: Renderer,
{
    states: Vec<T>,
    // Vecs keep a placeholder because they have the potential to add additional items,
    // after their own items but before the next neighbor. It is much easier to add an
    // item before a known placeholder than to add it after the last known item, so we
    // just leave a placeholder here unlike zero-or-one iterators (Option, Result, etc.)
    marker: R::Placeholder,
}

impl<T, R> Mountable<R> for VecState<T, R>
where
    T: Mountable<R>,
    R: Renderer,
{
    fn unmount(&mut self) {
        for state in self.states.iter_mut() {
            state.unmount();
        }
        self.marker.unmount();
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        for state in self.states.iter_mut() {
            state.mount(parent, marker);
        }
        self.marker.mount(parent, marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        if let Some(first) = self.states.first() {
            first.insert_before_this(child)
        } else {
            false
        }
    }
}

impl<T, R, const N: usize> Render<R> for [T; N]
where
    T: Render<R>,
    R: Renderer,
{
    type State = ArrayState<T::State, R, N>;

    fn build(self) -> Self::State {
        Self::State {
            states: self.map(T::build),
            _phantom: Default::default(),
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        let Self::State { states, .. } = state;
        let old = states;
        // this is an unkeyed diff
        self.into_iter()
            .zip(old.iter_mut())
            .for_each(|(new, old)| T::rebuild(new, old));
    }
}

/// Retained view state for a `Vec<_>`.
pub struct ArrayState<T, R, const N: usize>
where
    T: Mountable<R>,
    R: Renderer,
{
    states: [T; N],
    _phantom: PhantomData<R>,
}

impl<T, R, const N: usize> Mountable<R> for ArrayState<T, R, N>
where
    T: Mountable<R>,
    R: Renderer,
{
    fn unmount(&mut self) {
        self.states.iter_mut().for_each(Mountable::unmount);
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        for state in self.states.iter_mut() {
            state.mount(parent, marker);
        }
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        if let Some(first) = self.states.first() {
            first.insert_before_this(child)
        } else {
            false
        }
    }
}
