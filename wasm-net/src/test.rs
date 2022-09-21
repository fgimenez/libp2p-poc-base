use futures::prelude::*;
use libp2p::ping::{Event as PingEvent, Success as PingEventSuccess};
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p_wasm_ext::{ffi, ExtTransport};
use once_cell::unsync::OnceCell;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::task::Poll;
use wasm_bindgen_test::wasm_bindgen_test;

thread_local! {
     static LOGGER: OnceCell<()> = OnceCell::new();
}

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn browser_desktop_base() {
    LOGGER.with(|cell| {
        cell.get_or_init(|| wasm_logger::init(wasm_logger::Config::default()));
    });

    let transport = ExtTransport::new(ffi::websocket_transport());
    let mut swarm = crate::service(
        Some(transport),
        Some(String::from("/ip4/127.0.0.1/tcp/38615/ws")),
    );

    let (sender, receiver) = mpsc::channel();
    future::poll_fn(|cx| loop {
        match swarm.poll_next_unpin(cx) {
            Poll::Ready(Some(event)) => match event {
                SwarmEvent::Behaviour(PingEvent { result, .. }) => match result {
                    Ok(PingEventSuccess::Ping { .. }) => {
                        sender.send(1).unwrap();
                        return Poll::Ready(());
                    }
                    _ => {}
                },
                _ => {}
            },
            Poll::Ready(None) => return Poll::Ready(()),
            Poll::Pending => return Poll::Pending,
        }
    })
    .await;

    let mut total = 0;
    for _ in receiver {
        total += 1;
    }

    assert_eq!(total, 1);
}
