use tokio::net::{TcpStream, TcpListener};
use tokio::sync::{mpsc, broadcast};
use tokio_util::codec::{Framed};
use futures_util::{SinkExt, StreamExt};

use std::io;
use std::collections::HashMap;



#[tokio::main]
async fn main () {
    let listener = TcpListener::bind("127.0.0.1:7878").await;
}

// need to add
// 1. user join handling
// 2. user disconnect handling
// 3. broadcast channels
// 4. users that have joined
// 5. add the user disconnected to the protocol
