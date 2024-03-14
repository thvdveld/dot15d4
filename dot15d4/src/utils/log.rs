//! Logger backend agnostic logging

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        #[cfg(all(not(feature="log"), feature = "defmt"))]
        defmt::error!($($arg)*);
        #[cfg(feature = "log")]
        ::log::error!($($arg)*);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        #[cfg(all(not(feature="log"), feature = "defmt"))]
        defmt::warn!($($arg)*);
        #[cfg(feature = "log")]
        ::log::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        #[cfg(all(not(feature="log"), feature = "defmt"))]
        ::defmt::info!($($arg)*);
        #[cfg(feature = "log")]
        ::log::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(all(not(feature="log"), feature = "defmt"))]
        ::defmt::debug!($($arg)*);
        #[cfg(feature = "log")]
        ::log::debug!($($arg)*);
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        #[cfg(all(not(feature="log"), feature = "defmt"))]
        ::defmt::trace!($($arg)*);
        #[cfg(feature = "log")]
        ::log::trace!($($arg)*);
    };
}
