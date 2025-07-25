use super::{
    any_view::{AnyView, IntoAny},
    iterators::StaticVec,
};
use crate::prelude::Renderer;

/// A typed-erased collection of different views.
pub struct Fragment<R>
where
    R: Renderer,
{
    /// The nodes contained in the fragment.
    pub nodes: StaticVec<AnyView<R>>,
}

/// Converts some view into a type-erased collection of views.
pub trait IntoFragment<R>
where
    R: Renderer,
{
    /// Converts some view into a type-erased collection of views.
    fn into_fragment(self) -> Fragment<R>;
}

impl<R> FromIterator<AnyView<R>> for Fragment<R>
where
    R: Renderer,
{
    fn from_iter<T: IntoIterator<Item = AnyView<R>>>(iter: T) -> Self {
        Fragment::new(iter.into_iter().collect())
    }
}

impl<R> From<AnyView<R>> for Fragment<R>
where
    R: Renderer,
{
    fn from(view: AnyView<R>) -> Self {
        Fragment::new(vec![view])
    }
}

impl<R> From<Fragment<R>> for AnyView<R>
where
    R: Renderer,
{
    fn from(value: Fragment<R>) -> Self {
        value.nodes.into_any()
    }
}

impl<R> Fragment<R>
where
    R: Renderer,
{
    /// Creates a new [`Fragment`].
    #[inline(always)]
    pub fn new(nodes: Vec<AnyView<R>>) -> Self {
        Self {
            nodes: nodes.into(),
        }
    }
}

impl<R> IntoFragment<R> for AnyView<R>
where
    R: Renderer,
{
    fn into_fragment(self) -> Fragment<R> {
        Fragment::new(vec![self])
    }
}

impl<T, R> IntoFragment<R> for Vec<T>
where
    T: IntoAny<R>,
    R: Renderer,
{
    fn into_fragment(self) -> Fragment<R> {
        Fragment::new(self.into_iter().map(IntoAny::into_any).collect())
    }
}

impl<T, R> IntoFragment<R> for StaticVec<T>
where
    T: IntoAny<R>,
    R: Renderer,
{
    fn into_fragment(self) -> Fragment<R> {
        Fragment::new(self.into_iter().map(IntoAny::into_any).collect())
    }
}

impl<const N: usize, T, R> IntoFragment<R> for [T; N]
where
    T: IntoAny<R>,
    R: Renderer,
{
    fn into_fragment(self) -> Fragment<R> {
        Fragment::new(self.into_iter().map(IntoAny::into_any).collect())
    }
}

macro_rules! tuples {
	($($ty:ident),*) => {
		impl<$($ty),*, Rndr> IntoFragment<Rndr> for ($($ty,)*)
		where
			$($ty: IntoAny<Rndr>),*,
                        Rndr: Renderer

		{
                    fn into_fragment(self) -> Fragment<Rndr> {
                        #[allow(non_snake_case)]
		        let ($($ty,)*) = self;
                        Fragment::new(vec![$($ty.into_any(),)*])
                    }
                }
    }
}

tuples!(A);
tuples!(A, B);
tuples!(A, B, C);
tuples!(A, B, C, D);
tuples!(A, B, C, D, E);
tuples!(A, B, C, D, E, F);
tuples!(A, B, C, D, E, F, G);
tuples!(A, B, C, D, E, F, G, H);
tuples!(A, B, C, D, E, F, G, H, I);
tuples!(A, B, C, D, E, F, G, H, I, J);
tuples!(A, B, C, D, E, F, G, H, I, J, K);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y,
    Z
);
