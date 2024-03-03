#![no_std]
//! A handful of executor independent synchronization primitives.
//! The goal is to provide some synchronization within 1 task between different parts of that task.
pub(crate) mod channel;
pub(crate) mod join;
pub(crate) mod mutex;
pub(crate) mod select;
pub(crate) mod yield_now;

#[cfg(test)]
pub(crate) mod test;

/// Type representing 2 possible outcomes/states
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(PartialEq)]
pub enum Either<T, S> {
    First(T),
    Second(S),
}

impl<T, S> Either<T, S> {
    pub fn is_first(&self) -> bool {
        matches!(self, Either::First(_))
    }

    pub fn is_second(&self) -> bool {
        matches!(self, Either::Second(_))
    }
}
