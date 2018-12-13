pub trait SizeStrategy {
    type ErrorType: Into<crate::NoserError> + ::std::fmt::Debug;

    fn dynamic() -> bool;
}

pub enum SizeKind {
    Dynamic,
    Exactly(crate::Ptr),
}

pub trait Sizable {
    type Strategy: SizeStrategy;

    fn read_size(_: &[u8]) -> ReadReturn<Self>;

    #[inline]
    fn static_size() -> crate::Ptr
    where
        Self::Strategy: SizeStrategy<ErrorType = NoError>,
    {
        Self::read_size(&[]).expect("Guaranteed to succeed since NoError has no variants.")
    }

    #[inline]
    fn size() -> SizeKind {
        if Self::Strategy::dynamic() {
            SizeKind::Dynamic
        } else {
            SizeKind::Exactly(
                Self::read_size(&[])
                    .expect("Expect static strategies to never error on read size."),
            )
        }
    }

    #[inline]
    fn in_bounds(arena: &[u8]) -> crate::Result<crate::Ptr> {
        let size = Self::read_size(arena).map_err(Into::into)?;

        if arena.len() < size as usize {
            return Err(crate::NoserError::Undersized(size as usize, arena.to_owned()));
        }

        Ok(size)
    }
}

#[derive(Debug)]
pub enum NoError {}

#[allow(type_alias_bounds)]
pub type ReadReturn<T: Sizable> = Result<crate::Ptr, <T::Strategy as SizeStrategy>::ErrorType>;

impl Into<crate::NoserError> for NoError {
    fn into(self) -> crate::NoserError {
        unreachable!()
    }
}

pub struct Static;
pub struct Dynamic;

impl SizeStrategy for Static {
    type ErrorType = NoError;

    fn dynamic() -> bool {
        false
    }
}

impl SizeStrategy for Dynamic {
    type ErrorType = crate::NoserError;

    fn dynamic() -> bool {
        true
    }
}
