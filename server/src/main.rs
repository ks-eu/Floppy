use std::sync::Arc;

use tokio::sync::{
    RwLock,
    mpsc,
    oneshot,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed};
use futures_util::{StreamExt, SinkExt};

use flpc::tokio::{FLPCodec, FLPCMsg};

use crate::users::{
    UserState,
    try_set_sender,
};
use crate::login::accept::{
    JoinOutcome,
    handle_join,
    send_joinfail,
};
use crate::message::event::{
    ServerEvent,
    WriterResult,
    ClientEvent,
};

mod login;
mod error;
mod users;
mod message;

const MAXUSERS: u16 = 256;
const MAXUSERNAMESIZE: u8 = 64;
const PORT: &str = "127.0.0.1:7878";
const MAINCAPACITY: usize = 2048;
const CLIENTCAPACITY: usize = 128;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let users = Arc::new(RwLock::new(UserState::new(MAXUSERS as usize)?));
    let listener = TcpListener::bind(PORT).await?;
    let (main_tx, mut main_rx) = mpsc::channel::<ClientEvent>(MAINCAPACITY);

    // Login Handling
    let users_accept = users.clone();
    tokio::spawn(async move {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(stream) => stream,
                Err(_) => break,
            };
            
            // Handles initial login, onboarding and disconnects
            let users_conn = users_accept.clone();
            let client_tx = main_tx.clone();
            tokio::spawn(async move {
                let mut stream = Framed::new(stream, FLPCodec::new());
                let users_initconn = users_conn.clone();

                let id = match handle_join(
                    users_initconn,
                    &mut stream,
                    MAXUSERNAMESIZE,
                ).await {
                    JoinOutcome::Success(id) => id,
                    JoinOutcome::ClientFail => {
                        match send_joinfail(&mut stream, 0).await {
                            _ => return
                        } 
                    }
                    JoinOutcome::TcpFail => return,
                };

                let (mut sink, mut stream) = stream.split();

                let (tx, rx) = oneshot
                    ::channel
                    ::<WriterResult>();
                tokio::spawn(async move {
                    let (writer_tx, mut writer_rx) = mpsc
                        ::channel
                        ::<ServerEvent>(CLIENTCAPACITY);
                    if let Err(_) = tx.send(Ok(writer_tx)) {
                        return
                    }

                    while let Some(message) = writer_rx.recv().await {}
                    
                    return
                });

                let writer_tx = match rx.await {
                    Ok(Ok(tx)) => tx,
                    Ok(Err(_)) | Err(_) => {
                        let _ = send_joinfail(&mut sink, 0).await;
                        return;
                    }
                };
                 
                let users_setsender = users_conn.clone();
                if let Err(_) = try_set_sender(
                    users_setsender,
                    id,
                    writer_tx
                ).await {
                    return
                }

                let reader_tx = client_tx.clone();
                tokio::spawn(async move {
                    while let Some(msg) = stream.next().await {}
                });
            });
        }
    });

    Ok(())
}
