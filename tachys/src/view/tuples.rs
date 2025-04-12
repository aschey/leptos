use super::{Mountable, Render};
use crate::renderer::Renderer;

impl<R> Render<R> for ()
where
    R: Renderer,
{
    type State = R::Placeholder;

    fn build(self) -> Self::State {
        R::create_placeholder()
    }

    fn rebuild(self, _state: &mut Self::State) {}
}

impl<R> Mountable<R> for ()
where
    R: Renderer,
{
    fn unmount(&mut self) {}

    fn mount(&mut self, _parent: &R::Element, _marker: Option<&R::Node>) {}

    fn insert_before_this(&self, _child: &mut dyn Mountable<R>) -> bool {
        false
    }
}

impl<R: Renderer, A: Render<R>> Render<R> for (A,) {
    type State = A::State;

    fn build(self) -> Self::State {
        self.0.build()
    }

    fn rebuild(self, state: &mut Self::State) {
        self.0.rebuild(state)
    }
}

macro_rules! impl_view_for_tuples {
	($first:ident, $($ty:ident),* $(,)?) => {
		impl<$first, $($ty),*, Rndr> Render<Rndr> for ($first, $($ty,)*)
		where
			$first: Render<Rndr>,
			$($ty: Render<Rndr>),*,
            Rndr: Renderer,
		{
			type State = ($first::State, $($ty::State,)*);

			fn build(self) -> Self::State {
                #[allow(non_snake_case)]
                let ($first, $($ty,)*) = self;
                (
                    $first.build(),
                    $($ty.build()),*
                )
			}

			fn rebuild(self, state: &mut Self::State) {
				paste::paste! {
					let ([<$first:lower>], $([<$ty:lower>],)*) = self;
					let ([<view_ $first:lower>], $([<view_ $ty:lower>],)*) = state;
					[<$first:lower>].rebuild([<view_ $first:lower>]);
					$([<$ty:lower>].rebuild([<view_ $ty:lower>]));*
				}
			}


		}

        impl<$first, $($ty),*, Rndr> Mountable<Rndr> for ($first, $($ty,)*) where
			$first: Mountable<Rndr>,
			$($ty: Mountable<Rndr>),*,
			Rndr: Renderer
		{
			fn unmount(&mut self) {
                #[allow(non_snake_case)] // better macro performance
                let ($first, $($ty,)*) = self;
                $first.unmount();
                $($ty.unmount());*
			}

			fn mount(
				&mut self,
				parent: &Rndr::Element,
				marker: Option<&Rndr::Node>,
			) {
                #[allow(non_snake_case)] // better macro performance
                let ($first, $($ty,)*) = self;
                $first.mount(parent, marker);
                $($ty.mount(parent, marker));*
			}

			fn insert_before_this(&self,
				child: &mut dyn Mountable<Rndr>,
			) -> bool {
                #[allow(non_snake_case)] // better macro performance
                let ($first, $($ty,)*) = self;
                $first.insert_before_this(child)
                $(|| $ty.insert_before_this(child))*
			}
		}
    };
}

impl_view_for_tuples!(A, B);
impl_view_for_tuples!(A, B, C);
impl_view_for_tuples!(A, B, C, D);
impl_view_for_tuples!(A, B, C, D, E);
impl_view_for_tuples!(A, B, C, D, E, F);
impl_view_for_tuples!(A, B, C, D, E, F, G);
impl_view_for_tuples!(A, B, C, D, E, F, G, H);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T
);
impl_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U
);
impl_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V
);
impl_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W
);
impl_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
impl_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
impl_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y,
    Z
);
