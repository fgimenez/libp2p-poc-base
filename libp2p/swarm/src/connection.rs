// Copyright 2020 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

mod error;
mod handler_wrapper;

pub(crate) mod pool;

pub use error::{
    ConnectionError, PendingConnectionError, PendingInboundConnectionError,
    PendingOutboundConnectionError,
};
pub use pool::{ConnectionCounters, ConnectionLimits};
pub use pool::{EstablishedConnection, PendingConnection};

use crate::handler::ConnectionHandler;
use crate::IntoConnectionHandler;
use handler_wrapper::HandlerWrapper;
use libp2p_core::connection::ConnectedPoint;
use libp2p_core::multiaddr::Multiaddr;
use libp2p_core::muxing::{StreamMuxerBox, StreamMuxerEvent, StreamMuxerExt};
use libp2p_core::upgrade;
use libp2p_core::PeerId;
use std::collections::VecDeque;
use std::future::Future;
use std::{error::Error, fmt, io, pin::Pin, task::Context, task::Poll};

/// Information about a successfully established connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connected {
    /// The connected endpoint, including network address information.
    pub endpoint: ConnectedPoint,
    /// Information obtained from the transport.
    pub peer_id: PeerId,
}

/// Endpoint for a received substream.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubstreamEndpoint<TDialInfo> {
    Dialer(TDialInfo),
    Listener,
}

/// Event generated by a [`Connection`].
#[derive(Debug, Clone)]
pub enum Event<T> {
    /// Event generated by the [`ConnectionHandler`].
    Handler(T),
    /// Address of the remote has changed.
    AddressChange(Multiaddr),
}

/// A multiplexed connection to a peer with an associated [`ConnectionHandler`].
pub struct Connection<THandler>
where
    THandler: ConnectionHandler,
{
    /// Node that handles the muxing.
    muxing: StreamMuxerBox,
    /// Handler that processes substreams.
    handler: HandlerWrapper<THandler>,
    /// List of "open_info" that is waiting for new outbound substreams.
    open_info: VecDeque<handler_wrapper::OutboundOpenInfo<THandler>>,
}

impl<THandler> fmt::Debug for Connection<THandler>
where
    THandler: ConnectionHandler + fmt::Debug,
    THandler::OutboundOpenInfo: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Connection")
            .field("handler", &self.handler)
            .field("open_info", &self.open_info)
            .finish()
    }
}

impl<THandler> Unpin for Connection<THandler> where THandler: ConnectionHandler {}

impl<THandler> Connection<THandler>
where
    THandler: ConnectionHandler,
{
    /// Builds a new `Connection` from the given substream multiplexer
    /// and connection handler.
    pub fn new(
        peer_id: PeerId,
        endpoint: ConnectedPoint,
        muxer: StreamMuxerBox,
        handler: impl IntoConnectionHandler<Handler = THandler>,
        substream_upgrade_protocol_override: Option<upgrade::Version>,
        max_negotiating_inbound_streams: usize,
    ) -> Self {
        let wrapped_handler = HandlerWrapper::new(
            peer_id,
            endpoint,
            handler,
            substream_upgrade_protocol_override,
            max_negotiating_inbound_streams,
        );
        Connection {
            muxing: muxer,
            handler: wrapped_handler,
            open_info: VecDeque::with_capacity(8),
        }
    }

    /// Notifies the connection handler of an event.
    pub fn inject_event(&mut self, event: THandler::InEvent) {
        self.handler.inject_event(event);
    }

    /// Begins an orderly shutdown of the connection, returning the connection
    /// handler and a `Future` that resolves when connection shutdown is complete.
    pub fn close(self) -> (THandler, impl Future<Output = io::Result<()>>) {
        (self.handler.into_connection_handler(), self.muxing.close())
    }

    /// Polls the handler and the substream, forwarding events from the former to the latter and
    /// vice versa.
    pub fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Event<THandler::OutEvent>, ConnectionError<THandler::Error>>> {
        loop {
            // Poll the handler for new events.
            match self.handler.poll(cx)? {
                Poll::Pending => {}
                Poll::Ready(handler_wrapper::Event::OutboundSubstreamRequest(user_data)) => {
                    self.open_info.push_back(user_data);
                    continue; // Poll handler until exhausted.
                }
                Poll::Ready(handler_wrapper::Event::Custom(event)) => {
                    return Poll::Ready(Ok(Event::Handler(event)));
                }
            }

            match self.muxing.poll_unpin(cx)? {
                Poll::Pending => {}
                Poll::Ready(StreamMuxerEvent::AddressChange(address)) => {
                    self.handler.inject_address_change(&address);
                    return Poll::Ready(Ok(Event::AddressChange(address)));
                }
            }

            if !self.open_info.is_empty() {
                match self.muxing.poll_outbound_unpin(cx)? {
                    Poll::Pending => {}
                    Poll::Ready(substream) => {
                        let user_data = self
                            .open_info
                            .pop_front()
                            .expect("`open_info` is not empty");
                        let endpoint = SubstreamEndpoint::Dialer(user_data);
                        self.handler.inject_substream(substream, endpoint);
                        continue; // Go back to the top, handler can potentially make progress again.
                    }
                }
            }

            match self.muxing.poll_inbound_unpin(cx)? {
                Poll::Pending => {}
                Poll::Ready(substream) => {
                    self.handler
                        .inject_substream(substream, SubstreamEndpoint::Listener);
                    continue; // Go back to the top, handler can potentially make progress again.
                }
            }

            return Poll::Pending; // Nothing can make progress, return `Pending`.
        }
    }
}

/// Borrowed information about an incoming connection currently being negotiated.
#[derive(Debug, Copy, Clone)]
pub struct IncomingInfo<'a> {
    /// Local connection address.
    pub local_addr: &'a Multiaddr,
    /// Address used to send back data to the remote.
    pub send_back_addr: &'a Multiaddr,
}

impl<'a> IncomingInfo<'a> {
    /// Builds the [`ConnectedPoint`] corresponding to the incoming connection.
    pub fn create_connected_point(&self) -> ConnectedPoint {
        ConnectedPoint::Listener {
            local_addr: self.local_addr.clone(),
            send_back_addr: self.send_back_addr.clone(),
        }
    }
}

/// Information about a connection limit.
#[derive(Debug, Clone)]
pub struct ConnectionLimit {
    /// The maximum number of connections.
    pub limit: u32,
    /// The current number of connections.
    pub current: u32,
}

impl fmt::Display for ConnectionLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.current, self.limit)
    }
}

/// A `ConnectionLimit` can represent an error if it has been exceeded.
impl Error for ConnectionLimit {}