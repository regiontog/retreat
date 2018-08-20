use super::{Read, StaticSize};
use ext::SliceExt;
use implementation::ListLen;
use Ptr;

use std::marker::PhantomData;

pub const LTP_SIZE: Ptr = ::std::mem::size_of::<Ptr>() as Ptr;

pub struct StaticFind<T> {
    p: PhantomData<T>,
}

pub struct DynamicFind;

pub trait Strategy {
    fn find(&[u8], ListLen) -> Ptr;
    fn get_lookup_table(&mut [u8], ListLen) -> ::Result<(&[u8], &mut [u8])>;
}

pub trait Find {
    type Strategy: Strategy;

    #[inline]
    fn find(lookup_table: &[u8], idx: ListLen) -> Ptr {
        Self::Strategy::find(lookup_table, idx)
    }

    #[inline]
    fn get_lookup_table(arena: &mut [u8], capacity: ListLen) -> ::Result<(&[u8], &mut [u8])> {
        Self::Strategy::get_lookup_table(arena, capacity)
    }
}

impl<T: StaticSize> Strategy for StaticFind<T> {
    #[inline]
    fn find(_lookup_table: &[u8], idx: ListLen) -> Ptr {
        T::size() as ListLen * idx
    }

    #[inline]
    fn get_lookup_table(arena: &mut [u8], _capacity: ListLen) -> ::Result<(&[u8], &mut [u8])> {
        Ok((&[], arena))
    }
}

impl Strategy for DynamicFind {
    #[inline]
    fn find(lookup_table: &[u8], idx: ListLen) -> Ptr {
        if idx == 0 {
            0
        } else {
            Ptr::read(&lookup_table[::nth::<Ptr>(idx as usize - 1)])
        }
    }

    #[inline]
    fn get_lookup_table(arena: &mut [u8], capacity: ListLen) -> ::Result<(&[u8], &mut [u8])> {
        let (lookup_table, right) = arena.noser_split(capacity * LTP_SIZE)?;

        Ok((lookup_table, right))
    }
}
