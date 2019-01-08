pub trait NotDrop {}

impl<T: ?Sized> NotDrop for &T {}
impl<T: ?Sized> NotDrop for &mut T {}

pub struct AliasGuard<F, S> {
    first: std::mem::ManuallyDrop<F>,
    second: S,
}

impl<F, S> AliasGuard<F, S> {
    /// When AliasGuard is dropped F is not and S has their destructor ran normally!
    pub fn new(first: F, to_second: impl FnOnce(F) -> S) -> Self {
        let second = unsafe {
            let mut duplicate = std::mem::uninitialized();
            std::ptr::copy_nonoverlapping(&first, &mut duplicate, 1);
            to_second(duplicate)
        };

        AliasGuard {
            first: std::mem::ManuallyDrop::new(first),
            second,
        }
    }

    pub fn try_new<E>(first: F, to_second: impl FnOnce(F) -> Result<S, E>) -> Result<Self, (F, E)> {
        let second = unsafe {
            let mut duplicate = std::mem::uninitialized();
            std::ptr::copy_nonoverlapping(&first, &mut duplicate, 1);
            to_second(duplicate)
        };

        match second {
            Ok(second) => Ok(AliasGuard {
                first: std::mem::ManuallyDrop::new(first),
                second,
            }),
            Err(e) => Err((first, e)),
        }
    }

    pub fn first(&self) -> &F {
        &self.first
    }

    pub fn second(&self) -> &S {
        &self.second
    }

    pub fn mut_first(&mut self) -> &mut F {
        &mut self.first
    }

    pub fn mut_second(&mut self) -> &mut S {
        &mut self.second
    }

    pub fn move_first(self) -> F
    where
        F: NotDrop,
    {
        std::mem::ManuallyDrop::into_inner(self.first)
    }

    pub fn move_second(self) -> S {
        self.second
    }
}

// pub fn try_map<R, E>(
//     self,
//     func: impl FnOnce(F) -> Result<R, E>,
// ) -> Result<AliasGuard<F, R>, (Self, E)> {
//     match AliasGuard::try_new(self.first, func) {
//         Ok(guard) => Ok(guard),
//         Err(e) => Err((self, e)),
//     }
// }
