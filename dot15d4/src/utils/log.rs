//! Logger backend agnostic logging

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        #[cfg(feature = "log-defmt")]
        defmt::error!($($arg)*);
        #[cfg(feature = "log-tracing")]
        tracing::error!($($arg)*);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        #[cfg(feature = "log-defmt")]
        defmt::warn!($($arg)*);
        #[cfg(feature = "log-tracing")]
        tracing::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        #[cfg(feature = "log-defmt")]
        defmt::info!($($arg)*);
        #[cfg(feature = "log-tracing")]
        tracing::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "log-defmt")]
        defmt::debug!($($arg)*);
        #[cfg(feature = "log-tracing")]
        tracing::debug!($($arg)*);
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        #[cfg(feature = "log-defmt")]
        defmt::trace!($($arg)*);
        #[cfg(feature = "log-tracing")]
        tracing::trace!($($arg)*);
    };
}
