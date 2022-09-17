// Copyright 2017-2018 Parity Technologies (UK) Ltd.
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

//! This module implements the `/ipfs/ping/1.0.0` protocol.
//!
//! The ping protocol can be used as a simple application-layer health check
//! for connections of any [`Transport`] as well as to measure and record
//! round-trip times.
//!
//! # Usage
//!
//! The [`Ping`] struct implements the [`NetworkBehaviour`] trait. When used with a [`Swarm`],
//! it will respond to inbound ping requests and as necessary periodically send outbound
//! ping requests on every established connection. If a configurable number of consecutive
//! pings fail, the connection will be closed.
//!
//! The `Ping` network behaviour produces [`PingEvent`]s, which may be consumed from the `Swarm`
//! by an application, e.g. to collect statistics.
//!
//! > **Note**: The ping protocol does not keep otherwise idle connections alive
//! > by default, see [`PingConfig::with_keep_alive`] for changing this behaviour.
//!
//! [`Swarm`]: libp2p_swarm::Swarm
//! [`Transport`]: libp2p_core::Transport

mod handler;
mod protocol;

use handler::Handler;
pub use handler::{Config, Failure, Success};
use libp2p_core::{connection::ConnectionId, PeerId};
use libp2p_swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters};
use std::{
    collections::VecDeque,
    task::{Context, Poll},
};

#[deprecated(
    since = "0.30.0",
    note = "Use re-exports that omit `Ping` prefix, i.e. `libp2p::ping::Config` etc"
)]
pub use self::{
    protocol::PROTOCOL_NAME, Config as PingConfig, Event as PingEvent, Failure as PingFailure,
    Result as PingResult, Success as PingSuccess,
};
#[deprecated(since = "0.30.0", note = "Use libp2p::ping::Behaviour instead.")]
pub use Behaviour as Ping;

/// The result of an inbound or outbound ping.
pub type Result = std::result::Result<Success, Failure>;

/// A [`NetworkBehaviour`] that responds to inbound pings and
/// periodically sends outbound pings on every established connection.
///
/// See the crate root documentation for more information.
pub struct Behaviour {
    /// Configuration for outbound pings.
    config: Config,
    /// Queue of events to yield to the swarm.
    events: VecDeque<Event>,
}

/// Event generated by the `Ping` network behaviour.
#[derive(Debug)]
pub struct Event {
    /// The peer ID of the remote.
    pub peer: PeerId,
    /// The result of an inbound or outbound ping.
    pub result: Result,
}

impl Behaviour {
    /// Creates a new `Ping` network behaviour with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            events: VecDeque::new(),
        }
    }
}

impl Default for Behaviour {
    fn default() -> Self {
        Self::new(Config::new())
    }
}

impl NetworkBehaviour for Behaviour {
    type ConnectionHandler = Handler;
    type OutEvent = Event;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        Handler::new(self.config.clone())
    }

    fn inject_event(&mut self, peer: PeerId, _: ConnectionId, result: Result) {
        self.events.push_front(Event { peer, result })
    }

    fn poll(
        &mut self,
        _: &mut Context<'_>,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        if let Some(e) = self.events.pop_back() {
            let Event { result, peer } = &e;

            match result {
                Ok(Success::Ping { .. }) => log::debug!("Ping sent to {:?}", peer),
                Ok(Success::Pong) => log::debug!("Ping received from {:?}", peer),
                _ => {}
            }

            Poll::Ready(NetworkBehaviourAction::GenerateEvent(e))
        } else {
            Poll::Pending
        }
    }
}
