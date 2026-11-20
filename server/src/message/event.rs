use tokio::sync::mpsc;

use flpc::header::{
    Message,
    ClientMessage,
    UserJoin,
    UserLeft,
};

pub type WriterResult = Result<
    mpsc::Sender<ServerEvent>,
    Box<dyn std::error::Error + Send + Sync>
>;

pub enum ServerEvent {
    Message(ClientMessage),
    UserJoin(UserJoin),
    UserLeft(UserLeft),
}

pub enum ClientEvent {
    Message(Message),
    Connected(u16),
    Disconnected(u16),
}
