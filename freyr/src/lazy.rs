use crate::Either;

pub struct Lazy<T, F> {
    lazy: Either<T, F>,
}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    pub fn new(func: F) -> Self {
        Lazy {
            lazy: Either::Right(func),
        }
    }

    pub fn get(&mut self) -> &mut T {
        self.lazy.unify_left(|func| func())
    }
}
