//! Traits for defining generic RPC handlers.
use std::future::Future;

use bytes;
use failure;

use crate::descriptor;

/// An implementation of a specific RPC handler.
///
/// This can be an actual implementation of a service, or something that will send a request over
/// a network to fulfill a request.
pub trait Handler: Clone + Send + 'static {
    /// The type of errors that this handler might generate, beyond the default RPC error type.
    type Error: failure::Fail;
    /// The service descriptor for the service whose requests this handler can handle.
    type Descriptor: descriptor::ServiceDescriptor;
    /// The future that results from a call to the `call` method of this trait.
    type CallFuture: Future<Output = Result<bytes::Bytes, Self::Error>> + Send;

    /// Perform a raw call to the specified service and method.
    fn call(
        &self,
        method: <Self::Descriptor as descriptor::ServiceDescriptor>::Method,
        input: bytes::Bytes,
    ) -> Self::CallFuture;
}
