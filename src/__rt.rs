//! Utility functions used by generated code; this is *not* part of the crate's public API!
use futures::task::{Context, Poll};
use futures::Future;
use std::error::Error as StdError;
use std::marker::PhantomData;
use std::pin::Pin;

use pin_project::pin_project;

use crate::descriptor;
use crate::error;
use crate::handler;

/// A future returned by a client call.
#[derive(Debug)]
#[pin_project(project = ClientFutureProj, project_replace = ClientFutureReplace)]
pub enum ClientFuture<H, I, O>
where
    H: handler::Handler,
{
    /// The message has not yet been encoded.
    Encode(
        Option<I>,
        Option<H>,
        Option<<H::Descriptor as descriptor::ServiceDescriptor>::Method>,
    ),
    /// The message was sent over RPC but the call future is not yet done.
    Call(#[pin] H::CallFuture),
    /// We have returned the response to the caller.
    Done(PhantomData<O>),
}

impl<H, I, O> ClientFuture<H, I, O>
where
    H: handler::Handler,
    I: prost::Message,
    O: prost::Message + Default,
{
    pub fn new(
        handler: H,
        input: I,
        method: <H::Descriptor as descriptor::ServiceDescriptor>::Method,
    ) -> Self {
        ClientFuture::Encode(Some(input), Some(handler), Some(method))
    }
}

impl<H, I, O> Future for ClientFuture<H, I, O>
where
    H: handler::Handler,
    I: prost::Message,
    O: prost::Message + Default,
{
    type Output = Result<O, error::Error<H::Error>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            // Decide what to do next inside a block so the projection borrow ends
            enum Step<H, I, O>
            where
                H: handler::Handler,
                I: prost::Message,
                O: prost::Message + Default,
            {
                Replace(ClientFuture<H, I, O>),
                Return(Poll<<ClientFuture<H, I, O> as Future>::Output>),
            }

            let step = {
                match self.as_mut().project() {
                    ClientFutureProj::Encode(input, handler, method) => {
                        let input = input
                            .take()
                            .expect("Encode: polled after transition (input)");
                        let h = handler
                            .take()
                            .expect("Encode: polled after transition (handler)");
                        let m = method
                            .take()
                            .expect("Encode: polled after transition (method)");

                        let bytes = encode(input)?;
                        let fut = h.call(m, bytes);

                        Step::Replace(ClientFuture::Call(fut))
                    }

                    ClientFutureProj::Call(fut) => match fut.poll(cx) {
                        Poll::Pending => Step::Return(Poll::Pending),
                        Poll::Ready(Ok(bytes)) => {
                            let out = decode::<O, _>(bytes)?;
                            Step::Return(Poll::Ready(Ok(out)))
                        }
                        Poll::Ready(Err(e)) => {
                            Step::Return(Poll::Ready(Err(error::Error::execution(e))))
                        }
                    },

                    ClientFutureProj::Done(_) => panic!("polled after completion"),
                }
            };

            match step {
                Step::Replace(new_state) => {
                    let _old: ClientFutureReplace<_, _, _> =
                        self.as_mut().project_replace(new_state);
                    continue;
                }
                Step::Return(Poll::Pending) => return Poll::Pending,
                Step::Return(Poll::Ready(res)) => {
                    let _old = self
                        .as_mut()
                        .project_replace(ClientFuture::Done(PhantomData));
                    return Poll::Ready(res);
                }
            }
        }
    }
}

/// Efficiently decode a particular message type from a byte buffer.
pub fn decode<M, E>(buf: bytes::Bytes) -> error::Result<M, E>
where
    M: prost::Message + Default,
    E: StdError,
{
    let message = prost::Message::decode(buf)?;
    Ok(message)
}

/// Efficiently encode a particular message into a byte buffer.
pub fn encode<M, E>(message: M) -> error::Result<bytes::Bytes, E>
where
    M: prost::Message,
    E: StdError,
{
    let len = prost::Message::encoded_len(&message);
    let mut buf = ::bytes::BytesMut::with_capacity(len);
    prost::Message::encode(&message, &mut buf)?;
    Ok(buf.freeze())
}
