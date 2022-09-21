use std::str::FromStr;

use wasm_bindgen::prelude::*;

use futures::prelude::*;
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p_wasm_ext::{ffi, ExtTransport};
use std::task::Poll;

pub use console_error_panic_hook::set_once as set_console_error_panic_hook;
pub use console_log::init_with_level as init_console_log;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("hello from rust-wasm");
}

#[wasm_bindgen]
pub struct Client {}

/// Starts the client.
#[wasm_bindgen]
pub async fn start_client(dial: String, log_level: String) -> Result<Client, JsValue> {
    start_inner(dial, log_level)
        .await
        .map_err(|err| JsValue::from_str(&err.to_string()))
}

async fn start_inner(
    dial: String,
    log_level: String,
) -> Result<Client, Box<dyn std::error::Error>> {
    console_error_panic_hook::set_once();
    init_console_log(log::Level::from_str(&log_level)?)?;

    wasm_bindgen_futures::spawn_local(run(dial));

    Ok(Client {})
}

fn run(dial: String) -> impl Future<Output = ()> {
    let transport = ExtTransport::new(ffi::websocket_transport());
    let mut swarm = crate::service(Some(transport), Some(dial));

    future::poll_fn(move |cx| loop {
        match swarm.poll_next_unpin(cx) {
            Poll::Ready(Some(event)) => match event {
                SwarmEvent::Behaviour(event) => log::info!("{:?}", event),
                _ => {}
            },
            Poll::Ready(None) => return Poll::Ready(()),
            Poll::Pending => return Poll::Pending,
        }
    })
}
