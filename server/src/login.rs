use std::io::{self, ErrorKind};
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::{oneshot, RwLock};
use tokio_util::codec::Framed;
use futures_util::{StreamExt, SinkExt};

use flpc::header::JoinFail;
use flpc::tokio::{FLPCodec, FLPCMsg};
use flpc::error::ProtocolError;

use crate::error::UsersError;
use crate::state::Users;


pub async fn handle_login(
    users: Arc<RwLock<Users>>,
    stream: TcpStream,
    sender: oneshot::Sender<(Framed<TcpStream, FLPCodec>, u16)>,
    max_length: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut flpc_stream = Framed::new(stream, FLPCodec::new());
    let first = match flpc_stream.next().await {
        Some(Ok(FLPCMsg::Join(username))) => username,

        Some(Ok(_))
        | Some(Err(
            ProtocolError::UnknownHeader | ProtocolError::UTF8Error,
        )) => {
            send_join_fail(&mut flpc_stream, 0).await?;
            return Ok(());
        }

        Some(Err(ProtocolError::IoError(error))) => {
            if matches!(
                error.kind(),
                ErrorKind::ConnectionReset
                    | ErrorKind::ConnectionAborted
                    | ErrorKind::BrokenPipe
            ) {
                return Ok(());
            }

            if error.kind() == ErrorKind::Other
                && error.to_string() == "bytes remaining on stream"
            {
                return Ok(());
            }
            return Err(error.into());
        }

        Some(Err(error)) => {
            return Err(io::Error::other(format!(
                "failed to read login frame: {error:?}"
            ))
            .into());
        }
        None => return Ok(()),
    };

    if first.length() > max_length {
        send_join_fail(&mut flpc_stream, 0).await?;
        return Ok(());
    }

    let mut users_guard = users.write().await;
    let result = users_guard.insert(first.username());
    drop(users_guard);

    let id = match result {
        Ok(id) => id,
        Err(UsersError::AtCapacity | UsersError::DuplicateUsername) => {
            send_join_fail(&mut flpc_stream, 0).await?;
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    
    if let Err(_) = sender.send((flpc_stream, id)) {
        let mut users_guard = users.write().await;
        if let Err(e) = users_guard.remove(id) {
            panic!("CRITICAL ERROR: {e}")
        }
        drop(users_guard);
        return Ok(());
    }
    Ok(()) 
}


async fn send_join_fail(
    stream: &mut Framed<TcpStream, FLPCodec>,
    failcode: u8,
) -> io::Result<()> {
    stream
        .send(FLPCMsg::JoinFail(JoinFail::new(failcode)))
        .await
        .map_err(|error| match error {
            ProtocolError::IoError(error) => error,
            error => io::Error::other(format!(
                "failed to send JoinFail: {error:?}"
            )),
        })
}
