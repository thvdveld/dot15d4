//! A handful of executor independent synchronization primitives.
//! The goal is to provide some synchronization within 1 task between different parts of that task.
pub(crate) mod channel;
pub(crate) mod join;
pub(crate) mod mutex;
pub(crate) mod select;
pub(crate) mod yield_now;

#[cfg(test)]
pub(crate) mod tests;

/// Type representing 2 possible outcomes/states
#[derive(Debug, PartialEq)]
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
