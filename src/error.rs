//! Error type definitions for errors that can occur during RPC interactions.
use std::error::Error as StdError;

use prost;

/// A convenience type alias for creating a `Result` with the error being of type `Error`.
pub type Result<A, E> = std::result::Result<A, Error<E>>;

/// An error has occurred.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum Error<E>
where
    E: StdError,
{
    /// An error occurred during the execution of a (server) RPC endpoint or a (client) RPC transfer
    /// mechanism.
    #[error("Execution error: {error}")]
    Execution {
        /// The underlying execution error.
        #[source]
        error: E,
    },
    /// An error occurred during input decoding.
    #[error("Decode error: {error}")]
    Decode {
        /// The underlying decode error.
        #[source]
        error: prost::DecodeError,
    },
    /// An error occurred during output encoding.
    #[error("Encode error: {error}")]
    Encode {
        /// The underlying encode error.
        #[source]
        error: prost::EncodeError,
    },
}

impl<E> Error<E>
where
    E: StdError,
{
    /// Constructs a new execution error.
    pub fn execution(error: E) -> Self {
        Error::Execution { error }
    }
}

impl<E> From<prost::DecodeError> for Error<E>
where
    E: StdError,
{
    fn from(error: prost::DecodeError) -> Self {
        Error::Decode { error }
    }
}

impl<E> From<prost::EncodeError> for Error<E>
where
    E: StdError,
{
    fn from(error: prost::EncodeError) -> Self {
        Error::Encode { error }
    }
}
