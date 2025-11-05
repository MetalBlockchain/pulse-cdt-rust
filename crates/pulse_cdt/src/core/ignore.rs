use core::marker::PhantomData;

use pulse_serialization::{NumBytes, Read};

/// A type that can be used to ignore parameters in action handlers.
/// Currently non-ignore types can not succeed an ignore type in a method definition, i.e. void foo(float, ignore<int>) is allowed and void foo(float, ignore<int>, int) is not allowed.
pub struct Ignore<T: NumBytes> {
    _data: PhantomData<T>
}

impl<T: NumBytes> NumBytes for Ignore<T> {
    #[inline(always)]
    fn num_bytes(&self) -> usize {
        0
    }
}

impl<T: NumBytes> Read for Ignore<T> {
    #[inline(always)]
    fn read(_bytes: &[u8], _pos: &mut usize) -> Result<Self, pulse_serialization::ReadError> {
        Ok(Ignore { _data: PhantomData })
    }
}