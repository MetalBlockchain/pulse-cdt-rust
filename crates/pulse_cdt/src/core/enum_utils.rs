use core::ops::BitAnd;

pub trait BitEnum {
    type Repr: Copy
        + Eq
        + Default             // for zero
        + BitAnd<Output = Self::Repr>;
    fn to_bits(self) -> Self::Repr;
}

#[inline]
pub fn has_field<E: BitEnum>(flags: E::Repr, field: E) -> bool {
    (flags & field.to_bits()) != E::Repr::default()
}