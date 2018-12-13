pub trait SizeStrategy {
    type ErrorType: Into<::NoserError> + ::std::fmt::Debug;

    fn dynamic() -> bool;
}

pub enum SizeKind {
    Dynamic,
    Exactly(::Ptr),
}

pub trait Sizable {
    type Strategy: SizeStrategy;

    fn read_size(&[u8]) -> ReadReturn<Self>;

    #[inline]
    fn static_size() -> ::Ptr
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
    fn in_bounds(arena: &[u8]) -> ::Result<::Ptr> {
        let size = Self::read_size(arena).map_err(Into::into)?;

        if arena.len() < size as usize {
            return Err(::NoserError::Undersized(size as usize, arena.to_owned()));
        }

        Ok(size)
    }
}

#[derive(Debug)]
pub enum NoError {}

#[allow(type_alias_bounds)]
pub type ReadReturn<T: Sizable> = Result<::Ptr, <T::Strategy as SizeStrategy>::ErrorType>;

impl Into<::NoserError> for NoError {
    fn into(self) -> ::NoserError {
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
    type ErrorType = ::NoserError;

    fn dynamic() -> bool {
        true
    }
}
