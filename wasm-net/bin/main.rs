use async_std::task;
use env_logger::{Builder, Env};
use futures::prelude::*;
use libp2p::swarm::{Swarm, SwarmEvent};
use std::task::Poll;

fn main() {
    Builder::from_env(Env::default().default_filter_or("info")).init();
    let to_dial = std::env::args().nth(1);

    let mut swarm = wasm_net::service(None, to_dial, None);

    let mut listening = false;

    task::block_on(future::poll_fn(move |cx| loop {
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
    }));
}
