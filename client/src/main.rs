use std::io;

use crossterm::event::EventStream;
use crossterm::execute;
use crossterm::terminal::{
    enable_raw_mode,
    disable_raw_mode,
    EnterAlternateScreen,
    LeaveAlternateScreen,
};

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use tokio::sync::mpsc;

use futures_util::StreamExt;

use client::chatwindow::ChatWindow;
use client::input::{
    InputEvent,
    keypress,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let (send, mut recv) = mpsc::channel::<InputEvent>(100);

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut chatwindow = ChatWindow::new(String::from("You"), 256, 200);

    let input_send = send.clone();
    tokio::spawn(async move {
        let mut events = EventStream::new();
        while let Some(event) = events.next().await {
            let input_event = match event {
                Ok(event) => keypress(event),
                Err(_) => {
                    let _ = input_send.send(InputEvent::Error).await;
                    break;
                }
            };

            let Some(input_event) = input_event else {
                continue;
            };

            if input_send.send(input_event).await.is_err() {
                break;
            }
        }
    });

    terminal.draw(|frame| {
        chatwindow.render(frame);
    })?;
    loop {
		match recv.recv().await {
			Some(InputEvent::KeyStroke(c)) => {
				chatwindow.add_input(c);
			}
			Some(InputEvent::BackSpace) => {
				chatwindow.remove_input();
			}
			Some(InputEvent::Enter) => {
				let output = chatwindow.send_input();
                let msg = format!("{}> {}", chatwindow.username(), output); 
                chatwindow.add_message(msg);
			}
			Some(InputEvent::Exit) => {
				chatwindow.quit();
				break;
			}
			Some(InputEvent::Error) => {
				break;
			}
			None => break,
		}

        terminal.draw(|frame| {
            chatwindow.render(frame);
        })?;

        
        if chatwindow.state() {
            break;
        }
    }

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;

    Ok(())
}
