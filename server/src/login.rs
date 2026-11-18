use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};

use crate::state::Users;


pub async fn login_handler(
    users: Arc<RwLock<Users>>,
    listener: TcpStream,
    sender: mpsc::Sender<TcpStream>
) -> Result<(), std::io::Error> {
    Ok(())
}
