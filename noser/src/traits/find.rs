use super::{Read, StaticSize, Write};
use ext::SliceExt;
use implementation::ListLen;
use Ptr;

use std::marker::PhantomData;

pub const LTP_SIZE: Ptr = ::std::mem::size_of::<Ptr>() as Ptr;

pub struct StaticFind<T> {
    phantom: PhantomData<T>,
}

pub struct DynamicFind;

pub trait Strategy {
    fn find(&[u8], ListLen) -> Ptr;
    fn get_lookup_table(&mut [u8], ListLen) -> ::Result<(&mut [u8], &mut [u8])>;
    fn write_lookup_ptr(&mut [u8], i: usize, Ptr);
}

pub trait Find {
    type Strategy: Strategy;

    #[inline]
    fn find(lookup_table: &[u8], idx: ListLen) -> Ptr {
        Self::Strategy::find(lookup_table, idx)
    }

    #[inline]
    fn get_lookup_table(arena: &mut [u8], capacity: ListLen) -> ::Result<(&mut [u8], &mut [u8])> {
        Self::Strategy::get_lookup_table(arena, capacity)
    }

    #[inline]
    fn write_lookup_ptr(lookup_table: &mut [u8], i: usize, ptr: Ptr) {
        Self::Strategy::write_lookup_ptr(lookup_table, i, ptr)
    }
}

impl<T: StaticSize> Strategy for StaticFind<T> {
    #[inline]
    fn find(_lookup_table: &[u8], idx: ListLen) -> Ptr {
        // VERIFY: no overflow
        T::size() as ListLen * idx
    }

    #[inline]
    fn get_lookup_table(arena: &mut [u8], _capacity: ListLen) -> ::Result<(&mut [u8], &mut [u8])> {
        Ok((&mut [], arena))
    }

    #[inline]
    fn write_lookup_ptr(_lookup_table: &mut [u8], _i: usize, _ptr: Ptr) {}
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
    fn get_lookup_table(arena: &mut [u8], capacity: ListLen) -> ::Result<(&mut [u8], &mut [u8])> {
        let (lookup_table, right) = arena.noser_split(
            capacity
                .checked_mul(LTP_SIZE)
                .ok_or(::NoserError::IntegerOverflow)?,
        )?;

        Ok((lookup_table, right))
    }

    #[inline]
    fn write_lookup_ptr(lookup_table: &mut [u8], i: usize, ptr: Ptr) {
        Ptr::write(&mut lookup_table[::nth::<Ptr>(i)], ptr)
    }
}
