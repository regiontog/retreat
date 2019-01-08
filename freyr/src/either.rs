pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn unify_left(&mut self, func: impl FnOnce(R) -> L) -> &mut L {
        let mut left = Either::Left(match self {
            Either::Left(v) => return v,
            Either::Right(right) => {
                let mut right_uninit: R = unsafe { std::mem::uninitialized() };
                std::mem::swap(right, &mut right_uninit);
                func(right_uninit)
            }
        });

        // Uninitialized value now inside self.
        std::mem::swap(self, &mut left);

        // Uninitialized value now inside left.
        std::mem::forget(left);

        match self {
            Either::Left(v) => v,
            Either::Right(_) => unreachable!(),
        }
    }
}
