//! A collection of nice macros for handling non fatal errors and
//! [None] values.

/// Shorthand for returning in a unit function and printing the error.
/// ```
/// fn test() {
///     error_return!(failable_method());
///     // If `failable_method` failed this wont run, and its error has been printed
/// }
/// ```
#[macro_export]
macro_rules! error_return {
    ($context:literal) => {{
        match $context {
            Ok(map) => map,
            Err(e) => {
                bevy::log::error!("{}:{}:{}: {e}", file!(), line!(), column!());
                return Default::default();
            }
        }
    }};
    ($context:expr) => {{
        match $context {
            Ok(map) => map,
            Err(e) => {
                bevy::log::error!("{}:{}:{}: {e}", file!(), line!(), column!());
                return Default::default();
            }
        }
    }};
}

/// Shorthand for continuing in a loop and printing the error.
/// ```
/// fn test() {
///     loop {
///         error_continue!(failable_method());
///         // If `failable_method` failed this wont run, and its error has been printed
///     }
/// }
/// ```
#[macro_export]
macro_rules! error_continue {
    ($context:literal) => {{
        match $context {
            Ok(map) => map,
            Err(e) => {
                bevy::log::error!("{}:{}:{}: {e}", file!(), line!(), column!());
                continue;
            }
        }
    }};
    ($context:expr) => {{
        match $context {
            Ok(map) => map,
            Err(e) => {
                bevy::log::error!("{}:{}:{}: {e}", file!(), line!(), column!());
                continue;
            }
        }
    }};
}

/// Shorthand for returning in a unit function if a function returns None.
/// ```
/// fn test() {
///     option_return!(nullable_method());
///     // If `nullable_method` returned `None` this wont run.
/// }
/// ```
#[macro_export]
macro_rules! option_return {
    ($context:literal) => {{
        match $context {
            Some(map) => map,
            None => {
                return Default::default();
            }
        }
    }};
    ($context:expr) => {{
        match $context {
            Some(map) => map,
            None => {
                return Default::default();
            }
        }
    }};
}

/// Shorthand for continuing in a loop function if a function returns None.
/// ```
/// fn test() {
///     loop {
///         option_return!(nullable_method());
///         // If `nullable_method` returned `None` this wont run.
///     }
/// }
/// ```
#[macro_export]
macro_rules! option_continue {
    ($context:literal) => {{
        match $context {
            Some(map) => map,
            None => {
                continue;
            }
        }
    }};
    ($context:expr) => {{
        match $context {
            Some(map) => map,
            None => {
                continue;
            }
        }
    }};
}
