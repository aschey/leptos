use super::{Mountable, Render};
use crate::prelude::Renderer;
use either_of::*;

impl<A, B, R> Render<R> for Either<A, B>
where
    A: Render<R>,
    B: Render<R>,
    R: Renderer,
{
    type State = Either<A::State, B::State>;

    fn build(self) -> Self::State {
        match self {
            Either::Left(left) => Either::Left(left.build()),
            Either::Right(right) => Either::Right(right.build()),
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        match (self, &mut *state) {
            (Either::Left(new), Either::Left(old)) => {
                new.rebuild(old);
            }
            (Either::Right(new), Either::Right(old)) => {
                new.rebuild(old);
            }
            (Either::Right(new), Either::Left(old)) => {
                let mut new_state = new.build();
                old.insert_before_this(&mut new_state);
                old.unmount();
                *state = Either::Right(new_state);
            }
            (Either::Left(new), Either::Right(old)) => {
                let mut new_state = new.build();
                old.insert_before_this(&mut new_state);
                old.unmount();
                *state = Either::Left(new_state);
            }
        }
    }
}

impl<A, B, R> Mountable<R> for Either<A, B>
where
    A: Mountable<R>,
    B: Mountable<R>,
    R: Renderer,
{
    fn unmount(&mut self) {
        match self {
            Either::Left(left) => left.unmount(),
            Either::Right(right) => right.unmount(),
        }
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        match self {
            Either::Left(left) => left.mount(parent, marker),
            Either::Right(right) => right.mount(parent, marker),
        }
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        match &self {
            Either::Left(left) => left.insert_before_this(child),
            Either::Right(right) => right.insert_before_this(child),
        }
    }
}

/// Stores each value in the view state, overwriting it only if `Some(_)` is provided.
pub struct EitherKeepAlive<A, B> {
    /// The first possibility.
    pub a: Option<A>,
    /// The second possibility.
    pub b: Option<B>,
    /// If `true`, then `b` will be shown.
    pub show_b: bool,
}

/// Retained view state for [`EitherKeepAlive`].
pub struct EitherKeepAliveState<A, B> {
    a: Option<A>,
    b: Option<B>,
    showing_b: bool,
}

impl<A, B, R> Render<R> for EitherKeepAlive<A, B>
where
    A: Render<R>,
    B: Render<R>,
    R: Renderer,
{
    type State = EitherKeepAliveState<A::State, B::State>;

    fn build(self) -> Self::State {
        let showing_b = self.show_b;
        let a = self.a.map(Render::build);
        let b = self.b.map(Render::build);
        EitherKeepAliveState { a, b, showing_b }
    }

    fn rebuild(self, state: &mut Self::State) {
        // set or update A -- `None` just means "no change"
        match (self.a, &mut state.a) {
            (Some(new), Some(old)) => new.rebuild(old),
            (Some(new), None) => state.a = Some(new.build()),
            _ => {}
        }

        // set or update B
        match (self.b, &mut state.b) {
            (Some(new), Some(old)) => new.rebuild(old),
            (Some(new), None) => state.b = Some(new.build()),
            _ => {}
        }

        match (self.show_b, state.showing_b) {
            // transition from A to B
            (true, false) => match (&mut state.a, &mut state.b) {
                (Some(a), Some(b)) => {
                    a.insert_before_this(b);
                    a.unmount();
                }
                _ => unreachable!(),
            },
            // transition from B to A
            (false, true) => match (&mut state.a, &mut state.b) {
                (Some(a), Some(b)) => {
                    b.insert_before_this(a);
                    b.unmount();
                }
                _ => unreachable!(),
            },
            _ => {}
        }
        state.showing_b = self.show_b;
    }
}

impl<A, B, R> Mountable<R> for EitherKeepAliveState<A, B>
where
    A: Mountable<R>,
    B: Mountable<R>,
    R: Renderer,
{
    fn unmount(&mut self) {
        if self.showing_b {
            self.b.as_mut().expect("B was not present").unmount();
        } else {
            self.a.as_mut().expect("A was not present").unmount();
        }
    }

    fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
        if self.showing_b {
            self.b
                .as_mut()
                .expect("B was not present")
                .mount(parent, marker);
        } else {
            self.a
                .as_mut()
                .expect("A was not present")
                .mount(parent, marker);
        }
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<R>) -> bool {
        if self.showing_b {
            self.b
                .as_ref()
                .expect("B was not present")
                .insert_before_this(child)
        } else {
            self.a
                .as_ref()
                .expect("A was not present")
                .insert_before_this(child)
        }
    }
}

macro_rules! tuples {
    ($num:literal => $($ty:ident),*) => {
        paste::paste! {
            #[doc = concat!("Retained view state for ", stringify!([<EitherOf $num>]), ".")]
            pub struct [<EitherOf $num State>]<$($ty,)* Rndr>
            where
                $($ty: Render<Rndr>,)*
                Rndr: Renderer

            {
                /// Which child view state is being displayed.
                pub state: [<EitherOf $num>]<$($ty::State,)*>,
            }

            impl<$($ty,)* Rndr> Mountable<Rndr> for [<EitherOf $num State>]<$($ty,)* Rndr>
            where
                $($ty: Render<Rndr>,)*
                Rndr: Renderer

            {
                fn unmount(&mut self) {
                    match &mut self.state {
                        $([<EitherOf $num>]::$ty(this) => [<EitherOf $num>]::$ty(this.unmount()),)*
                    };
                }

                fn mount(
                    &mut self,
                    parent: &Rndr::Element,
                    marker: Option<&Rndr::Node>,
                ) {
                    match &mut self.state {
                        $([<EitherOf $num>]::$ty(this) => [<EitherOf $num>]::$ty(this.mount(parent, marker)),)*
                    };
                }

                fn insert_before_this(&self,
                    child: &mut dyn Mountable<Rndr>,
                ) -> bool {
                    match &self.state {
                        $([<EitherOf $num>]::$ty(this) =>this.insert_before_this(child),)*
                    }
                }
            }

            impl<$($ty,)* Rndr> Render<Rndr> for [<EitherOf $num>]<$($ty,)*>
            where
                $($ty: Render<Rndr>,)*
                Rndr: Renderer

            {
                type State = [<EitherOf $num State>]<$($ty,)* Rndr>;

                fn build(self) -> Self::State {
                    let state = match self {
                        $([<EitherOf $num>]::$ty(this) => [<EitherOf $num>]::$ty(this.build()),)*
                    };
                    Self::State { state }
                }

                fn rebuild(self, state: &mut Self::State) {
                    let new_state = match (self, &mut state.state) {
                        // rebuild same state and return early
                        $(([<EitherOf $num>]::$ty(new), [<EitherOf $num>]::$ty(old)) => { return new.rebuild(old); },)*
                        // or mount new state
                        $(([<EitherOf $num>]::$ty(new), _) => {
                            let mut new = new.build();
                            state.insert_before_this(&mut new);
                            [<EitherOf $num>]::$ty(new)
                        },)*
                    };

                    // and then unmount old state
                    match &mut state.state {
                        $([<EitherOf $num>]::$ty(this) => this.unmount(),)*
                    };

                    // and store the new state
                    state.state = new_state;
                }
            }
        }
    }
}

tuples!(3 => A, B, C);
tuples!(4 => A, B, C, D);
tuples!(5 => A, B, C, D, E);
tuples!(6 => A, B, C, D, E, F);
tuples!(7 => A, B, C, D, E, F, G);
tuples!(8 => A, B, C, D, E, F, G, H);
tuples!(9 => A, B, C, D, E, F, G, H, I);
tuples!(10 => A, B, C, D, E, F, G, H, I, J);
tuples!(11 => A, B, C, D, E, F, G, H, I, J, K);
tuples!(12 => A, B, C, D, E, F, G, H, I, J, K, L);
tuples!(13 => A, B, C, D, E, F, G, H, I, J, K, L, M);
tuples!(14 => A, B, C, D, E, F, G, H, I, J, K, L, M, N);
tuples!(15 => A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
tuples!(16 => A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
