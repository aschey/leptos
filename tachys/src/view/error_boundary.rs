use crate::{
    prelude::Renderer,
    view::{iterators::OptionState, Mountable, Render},
};
use either_of::Either;
use std::sync::Arc;
use throw_error::{Error as AnyError, ErrorHook};

impl<T, E, R> Render<R> for Result<T, E>
where
    T: Render<R>,
    E: Into<AnyError> + 'static,
    R: Renderer,
{
    type State = ResultState<T, R>;

    fn build(self) -> Self::State {
        let hook = throw_error::get_error_hook();
        let (state, error) = match self {
            Ok(view) => (Either::Left(view.build()), None),
            Err(e) => (
                Either::Right(Render::<R>::build(())),
                Some(throw_error::throw(e.into())),
            ),
        };
        ResultState { state, error, hook }
    }

    fn rebuild(self, state: &mut Self::State) {
        let _guard = state.hook.clone().map(throw_error::set_error_hook);
        match (&mut state.state, self) {
            // both errors: throw the new error and replace
            (Either::Right(_), Err(new)) => {
                if let Some(old_error) = state.error.take() {
                    throw_error::clear(&old_error);
                }
                state.error = Some(throw_error::throw(new.into()));
            }
            // both Ok: need to rebuild child
            (Either::Left(old), Ok(new)) => {
                T::rebuild(new, old);
            }
            // Ok => Err: unmount, replace with marker, and throw
            (Either::Left(old), Err(err)) => {
                let mut new_state = Render::<R>::build(());
                old.insert_before_this(&mut new_state);
                old.unmount();
                state.state = Either::Right(new_state);
                state.error = Some(throw_error::throw(err));
            }
            // Err => Ok: clear error and build
            (Either::Right(old), Ok(new)) => {
                if let Some(err) = state.error.take() {
                    throw_error::clear(&err);
                }
                let mut new_state = new.build();
                old.insert_before_this(&mut new_state);
                old.unmount();
                state.state = Either::Left(new_state);
            }
        }
    }
}

/// View state for a `Result<_, _>` view.
pub struct ResultState<T, R>
where
    T: Render<R>,
    R: Renderer,
{
    /// The view state.
    state: OptionState<T, R>,
    error: Option<throw_error::ErrorId>,
    hook: Option<Arc<dyn ErrorHook>>,
}

impl<T, R> Drop for ResultState<T, R>
where
    T: Render<R>,
    R: Renderer,
{
    fn drop(&mut self) {
        // when the state is cleared, unregister this error; this item is being dropped and its
        // error should no longer be shown
        if let Some(e) = self.error.take() {
            throw_error::clear(&e);
        }
    }
}

impl<T, R> Mountable<R> for ResultState<T, R>
where
    T: Render<R>,
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
