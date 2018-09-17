#[macro_use]
extern crate log;

extern crate codechain_rpc as crpc;
extern crate jsonrpc_core;
extern crate primitives as cprimitives;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate ws;

#[macro_use]
mod logger;
mod frontend_handler;
mod jsonrpc;
mod rpc;

use std::cell::Cell;
use std::rc::Rc;

use ws::listen;

use self::frontend_handler::WebSocketHandler;
use self::logger::init as logger_init;
use self::rpc::router::Router;
use rpc::frontend::add_routing;
use std::sync::Arc;

fn main() {
    logger_init().expect("Logger should be initialized");

    let count = Rc::new(Cell::new(0));

    let mut router = Arc::new(Router::new());
    add_routing(Arc::get_mut(&mut router).unwrap());
    cinfo!("Listen on 3012 port");
    listen("127.0.0.1:3012", |out| WebSocketHandler {
        out,
        count: count.clone(),
        router: router.clone(),
    }).unwrap();
}
