//! Time structures.
//!
//! - [`Instant`] is used to represent a point in time.
//! - [`Duration`] is used to represent a duration of time.

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Instant {
    us: i64,
}

impl Instant {
    pub const fn from_us(us: i64) -> Self {
        Self { us }
    }

    pub const fn as_us(&self) -> i64 {
        self.us
    }
}

impl core::fmt::Display for Instant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:.2}ms", self.as_us() as f32 / 1000.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Duration(i64);

impl Duration {
    pub const fn from_us(us: i64) -> Self {
        Self(us)
    }

    pub const fn as_us(&self) -> i64 {
        self.0
    }
}

impl core::fmt::Display for Duration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:.2}ms", self.as_us() as f32 / 1000.0)
    }
}

impl core::ops::Sub for Instant {
    type Output = Self;

    fn sub(self, rhs: Instant) -> Self::Output {
        Self::from_us(self.as_us() - rhs.as_us())
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self::from_us(self.us - rhs.as_us())
    }
}

impl core::ops::Sub for Duration {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self::from_us(self.as_us() - rhs.as_us())
    }
}

impl core::ops::Mul<usize> for Duration {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        Self::from_us(self.as_us() * rhs as i64)
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self::from_us(self.us + rhs.as_us())
    }
}

impl core::ops::Div<usize> for Duration {
    type Output = Self;

    fn div(self, rhs: usize) -> Self::Output {
        Self::from_us(self.as_us() / rhs as i64)
    }
}

impl core::ops::Add<Duration> for Duration {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self::from_us(self.as_us() + rhs.as_us())
    }
}
