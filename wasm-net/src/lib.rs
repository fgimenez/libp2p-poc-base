#[cfg(feature = "browser")]
mod browser;

use futures::prelude::*;
use libp2p::core::transport::OptionalTransport;
use libp2p::multiaddr::Protocol;
use libp2p::ping::{Ping, PingConfig};
use libp2p::swarm::Swarm;
use libp2p::{
    core, identity, mplex, noise, swarm::SwarmEvent, wasm_ext, yamux, Multiaddr, PeerId, Transport,
};
use std::borrow::Cow;
use std::net::Ipv4Addr;
use std::task::Poll;

#[cfg(not(target_os = "unknown"))]
use libp2p::{dns, tcp, websocket};

#[cfg(not(target_os = "unknown"))]
use async_std::task;

// This is lifted from the rust libp2p-rs gossipsub and massaged to work with wasm.
// The "glue" to get messages from the browser injected into this service isn't done yet.
pub fn service(
    wasm_external_transport: Option<wasm_ext::ExtTransport>,
    dial: Option<String>,
) -> impl Future<Output = ()> {
    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = if let Some(t) = wasm_external_transport {
        OptionalTransport::some(t)
    } else {
        OptionalTransport::none()
    };

    #[cfg(not(target_os = "unknown"))]
    let transport = transport.or_transport({
        let ws_trans = websocket::WsConfig::new(tcp::TcpTransport::new(
            tcp::GenTcpConfig::new().nodelay(true),
        ))
        .or_transport(tcp::TcpTransport::new(
            tcp::GenTcpConfig::new().nodelay(true),
        ));

        task::block_on(dns::DnsConfig::system(ws_trans))
            .unwrap()
            .boxed()
    });

    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(&local_key)
        .expect("Signing libp2p-noise static DH keypair failed.");

    let transport: core::transport::Boxed<(PeerId, core::muxing::StreamMuxerBox)> = transport
        .upgrade(core::upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(core::upgrade::SelectUpgrade::new(
            yamux::YamuxConfig::default(),
            mplex::MplexConfig::default(),
        ))
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    // Create a Swarm to manage peers and events
    let mut swarm = {
        let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));

        libp2p::Swarm::new(transport, behaviour, local_peer_id)
    };

    // Listen on all interfaces on 38615.  Websockt can't receive incoming connections
    // on browser (oops?)
    #[cfg(not(target_os = "unknown"))]
    libp2p::Swarm::listen_on(
        &mut swarm,
        Multiaddr::empty()
            .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(38615))
            .with(Protocol::Ws(Cow::Borrowed("/"))),
    )
    .unwrap();

    if let Some(addr) = dial {
        let remote: Multiaddr = addr.parse().unwrap();
        swarm.dial(remote).unwrap();
        println!("Dialed {}", addr)
    }

    let mut listening = false;

    future::poll_fn(move |cx| loop {
        match swarm.poll_next_unpin(cx) {
            Poll::Ready(Some(event)) => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    log::info!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(event) => log::info!("{:?}", event),
                SwarmEvent::IncomingConnection {
                    local_addr,
                    send_back_addr,
                } => {
                    log::info!(
                        "Incoming connection local_addr: {} send_back_addr: {}",
                        local_addr,
                        send_back_addr
                    )
                }
                SwarmEvent::IncomingConnectionError {
                    local_addr,
                    send_back_addr,
                    error,
                } => {
                    log::info!(
                        "Incoming err local_addr: {} send_back_addr: {}, err: {}",
                        local_addr,
                        send_back_addr,
                        error
                    )
                }
                _ => {}
            },
            Poll::Ready(None) => return Poll::Ready(()),
            Poll::Pending => {
                if !listening {
                    for addr in Swarm::listeners(&swarm) {
                        log::info!("Listening on {}", addr);
                        listening = true;
                    }
                }
                return Poll::Pending;
            }
        }
    })
}
