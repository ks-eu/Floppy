use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use flpc::header::{ClientMessage, JoinFail, JoinSuccess, Message, UserJoin};
use flpc::tokio::{FLPCodec, FLPCMsg};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::Framed;

struct ServerState {
	next_uuid: u16,
	usernames: HashMap<u16, String>,
	clients: HashMap<u16, mpsc::Sender<FLPCMsg>>,
}

impl ServerState {
	fn new() -> Self {
		Self {
			next_uuid: 0,
			usernames: HashMap::new(),
			clients: HashMap::new(),
		}
	}

	fn allocate_uuid(&mut self) -> Option<u16> {
		if self.next_uuid > 255 {
			return None;
		}

		let uuid = self.next_uuid;
		self.next_uuid += 1;

		Some(uuid)
	}
}

type SharedState = Arc<Mutex<ServerState>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let listener = TcpListener::bind("127.0.0.1:7878").await?;
	let state = Arc::new(Mutex::new(ServerState::new()));

	println!("Server listening on 127.0.0.1:7878");

	loop {
		let (stream, address) = listener.accept().await?;
		let state = Arc::clone(&state);

		println!("Connection from {address}");

		tokio::spawn(async move {
			if let Err(error) = handle_client(stream, state).await {
				eprintln!("Client error: {error}");
			}
		});
	}
}

async fn handle_client(
	stream: TcpStream,
	state: SharedState,
) -> Result<(), Box<dyn Error + Send + Sync>> {
	let mut framed = Framed::new(stream, FLPCodec::new());

	let username = match framed.next().await {
		Some(Ok(FLPCMsg::Join(join))) => {
			println!("Received Join: username={}", join.username());
			join.username().to_string()
		}
		Some(Ok(_)) => {
			println!("Received unexpected message before login");
			return Ok(());
		}
		Some(Err(error)) => {
			eprintln!("Error receiving Join message: {error:?}");
			return Ok(());
		}
		None => {
			println!("Connection closed before Join message");
			return Ok(());
		}
	};

	let uuid = {
		let mut state = state.lock().await;
		state.allocate_uuid()
	};

	let uuid = match uuid {
		Some(uuid) => uuid,
		None => {
			println!("Sending JoinFail to {username}");

			framed
				.send(FLPCMsg::JoinFail(JoinFail::new(0)))
				.await
				.map_err(|error| format!("{error:?}"))?;

			return Ok(());
		}
	};

	println!("Sending JoinSuccess to {username} (UUID: {uuid})");

	framed
		.send(FLPCMsg::JoinSuccess(JoinSuccess {}))
		.await
		.map_err(|error| format!("{error:?}"))?;

	let (outbound_tx, mut outbound_rx) = mpsc::channel::<FLPCMsg>(256);

	{
		let mut state = state.lock().await;

		for (existing_uuid, existing_username) in &state.usernames {
			let user_join = UserJoin::new(existing_username, *existing_uuid)
				.map_err(|error| format!("{error:?}"))?;

			outbound_tx
				.send(FLPCMsg::UserJoin(user_join))
				.await
				.map_err(|error| format!("{error:?}"))?;

			println!(
				"Sending existing user to {username}: {} (UUID: {})",
				existing_username,
				existing_uuid
			);
		}

		for (existing_uuid, existing_client) in &state.clients {
			existing_client
				.send(FLPCMsg::UserJoin(
					UserJoin::new(&username, uuid)
						.map_err(|error| format!("{error:?}"))?
				))
				.await
				.map_err(|error| format!("{error:?}"))?;

			println!(
				"Broadcasting new user to UUID {}: {} (UUID: {})",
				existing_uuid,
				username,
				uuid
			);
		}

		state.usernames.insert(uuid, username.clone());
		state.clients.insert(uuid, outbound_tx);
	}

	println!("User joined: {username} (UUID: {uuid})");

	let (mut sink, mut stream) = framed.split();

	let writer_task = tokio::spawn(async move {
		while let Some(message) = outbound_rx.recv().await {
			match &message {
				FLPCMsg::ClientMessage(message) => {
					println!(
						"Sending ClientMessage to UUID {uuid}: sender={}, message={}",
						message.uuid(),
						message.msg()
					);
				}
				FLPCMsg::UserJoin(message) => {
					println!(
						"Sending UserJoin to UUID {uuid}: user={}, user UUID={}",
						message.username(),
						message.uuid()
					);
				}
				_ => {
					println!("Sending protocol message to UUID {uuid}");
				}
			}

			if sink.send(message).await.is_err() {
				eprintln!("Failed to send message to UUID {uuid}");
				break;
			}
		}
	});

	while let Some(result) = stream.next().await {
		match result {
			Ok(FLPCMsg::Message(message)) => {
				println!(
					"Received Message from UUID {uuid}: {}",
					message.msg()
				);

				route_message(uuid, message, &state).await;
			}
			Ok(FLPCMsg::Join(_)) => {
				println!("Received unexpected Join from UUID {uuid}");
			}
			Ok(FLPCMsg::JoinSuccess(_)) => {
				println!("Received unexpected JoinSuccess from UUID {uuid}");
			}
			Ok(FLPCMsg::JoinFail(_)) => {
				println!("Received unexpected JoinFail from UUID {uuid}");
			}
			Ok(FLPCMsg::ClientMessage(message)) => {
				println!(
					"Received unexpected ClientMessage from UUID {uuid}: sender={}, message={}",
					message.uuid(),
					message.msg()
				);
			}
			Ok(FLPCMsg::UserJoin(message)) => {
				println!(
					"Received unexpected UserJoin from UUID {uuid}: user UUID={}, username={}",
					message.uuid(),
					message.username()
				);
			}
			Err(error) => {
				eprintln!("Protocol error from UUID {uuid}: {error:?}");
				break;
			}
		}
	}

	{
		let mut state = state.lock().await;
		state.clients.remove(&uuid);
		state.usernames.remove(&uuid);
	}

	writer_task.abort();

	println!("User disconnected: {username} (UUID: {uuid})");

	Ok(())
}

async fn route_message(
	sender_uuid: u16,
	message: Message,
	state: &SharedState,
) {
	let recipients = {
		let state = state.lock().await;

		state
			.clients
			.iter()
			.filter(|(uuid, _)| **uuid != sender_uuid)
			.map(|(uuid, sender)| (*uuid, sender.clone()))
			.collect::<Vec<_>>()
	};

	for (recipient_uuid, recipient) in recipients {
		let client_message = match ClientMessage::new(sender_uuid, message.msg()) {
			Ok(message) => message,
			Err(error) => {
				eprintln!("Failed to construct ClientMessage: {error:?}");
				continue;
			}
		};

		println!(
			"Routing message from UUID {sender_uuid} to UUID {recipient_uuid}: {}",
			message.msg()
		);

		if recipient
			.send(FLPCMsg::ClientMessage(client_message))
			.await
			.is_err()
		{
			eprintln!("Failed to queue message for UUID {recipient_uuid}");
		}
	}
}
