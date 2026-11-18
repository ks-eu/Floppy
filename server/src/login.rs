use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tokio_util::codec::Framed;
use futures_util::StreamExt;

use flpc::header::*;
use flpc::tokio::{FLPCodec, FLPCMsg};
use flpc::error::ProtocolError;use crate::state::Users;


type ClientStream = Framed<TcpStream, FLPCodec>;

pub async fn handle_login(
    users: Arc<RwLock<Users>>,
    stream: TcpStream,
    sender: mpsc::Sender<ClientStream>
) -> Result<(), std::io::Error> {
    let mut flpc_stream = Framed::new(stream, FLPCodec::new());
        
    Ok(())
}
