use std::collections::VecDeque;
use std::collections::HashSet;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use unicode_width::UnicodeWidthChar;

use crate::input::InputEvent;

#[derive(Debug)]
pub struct ChatWindow {
    input: Vec<char>,
    messages: VecDeque<String>,
    users: HashSet<String>,
    username: String,
    i_limit: usize,
    m_limit: usize,
    quit: bool,
}

impl ChatWindow {
    pub fn new(username: String, i_limit: usize, m_limit: usize) -> Self {
        Self {
            input: Vec::new(),
            messages: VecDeque::new(),
            users: HashSet::new(),
            username: username,
            i_limit: i_limit,
            m_limit: m_limit,
            quit: false,
        }
    }

    // Input

    pub fn input(&self) -> &[char] {
        &self.input
    }
 
    pub fn add_input(&mut self, c: char) {
        if self.input.len() < self.i_limit {
            self.input.push(c);
        }
    }

    pub fn remove_input(&mut self) {
        self.input.pop();
    }

    pub fn send_input(&mut self) -> String {
        self.input.drain(..).collect::<String>()
    }

    // Messages

    pub fn messages(&self) -> &VecDeque<String> {
        &self.messages
    }

    pub fn add_message(&mut self, message: String) {
        if self.messages.len() >= self.m_limit {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }

    // Users

    pub fn users(&self) -> &HashSet<String> {
        &self.users
    }

    pub fn add_user(&mut self, username: &str) {
        self.users.insert(username.to_string()); 
    }

    pub fn remove_user(&mut self, username: &str) {
        self.users.remove(username);
    }

    // Username

    pub fn username(&self)  -> &str {
        &self.username
    }

    pub fn change_username(&mut self, username: &str) {
        self.username = String::from(username)
    }

    // Quit

    pub fn state(&self) -> bool {
        self.quit
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    // InputEvent Handling

    pub fn inputevent(&mut self, event: InputEvent) -> bool {
        match event {
            InputEvent::KeyStroke(c) => {
                self.add_input(c);
            }
            InputEvent::BackSpace => {
                self.remove_input();
            }
            InputEvent::Enter => {
                let output = self.send_input();
                let msg = format!("{}> {}", self.username(), output);
                self.add_message(msg);
            }
            InputEvent::Exit => {
                self.quit();
                return false;
            }
            InputEvent::Error => return false,
            _ => {}
        }
        true
    }

    // Rendering

	pub fn render(&self, frame: &mut Frame) {
		let area = frame.area();

		let horizontal = Layout::default()
			.direction(Direction::Horizontal)
			.constraints([
				Constraint::Percentage(80),
				Constraint::Percentage(20),
			])
			.split(area);

		let vertical = Layout::default()
			.direction(Direction::Vertical)
			.constraints([
				Constraint::Min(0),
				Constraint::Length(4),
			])
			.split(horizontal[0]);

		let output_block = Block::default()
			.title("Chat")
			.borders(Borders::ALL);

		let output_area = output_block.inner(vertical[0]);

		let messages: Vec<Line> = self.messages()
			.iter()
			.map(|message| Line::from(message.as_str()))
			.collect();

		let total_lines: usize = self.messages()
			.iter()
			.map(|message| {
				let width = output_area.width.max(1) as usize;

				message
					.lines()
					.map(|line| {
						let line_width: usize = line
							.chars()
							.map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
							.sum();

						line_width.div_ceil(width).max(1)
					})
					.sum::<usize>()
			})
			.sum();

		let visible_height = output_area.height as usize;
		let offset = visible_height.saturating_sub(total_lines);
		let scroll = total_lines.saturating_sub(visible_height);

		let output = Paragraph::new(messages)
			.wrap(Wrap { trim: false })
			.scroll((scroll as u16, 0));

		frame.render_widget(output_block, vertical[0]);

		let message_area = Rect::new(
			output_area.x,
			output_area.y + offset as u16,
			output_area.width,
			output_area.height - offset as u16,
		);

		frame.render_widget(output, message_area);

		let input_block = Block::default()
			.borders(Borders::ALL);

		let input_area = input_block.inner(vertical[1]);

		let input_string: String = self.input().iter().collect();

		let input_width = input_area.width as usize;

		if input_width > 0 {
			let total_width: usize = self.input()
				.iter()
				.map(|c| UnicodeWidthChar::width(*c).unwrap_or(0))
				.sum();

			let cursor_line = total_width / input_width;
			let cursor_column = total_width % input_width;

			let scroll = cursor_line.saturating_sub(1);

			let input = Paragraph::new(input_string)
				.wrap(Wrap { trim: false })
				.scroll((scroll as u16, 0));

			frame.render_widget(input_block, vertical[1]);
			frame.render_widget(input, input_area);

			frame.set_cursor_position(Position::new(
				input_area.x + cursor_column as u16,
				input_area.y + (cursor_line - scroll) as u16,
			));
		} else {
			frame.render_widget(input_block, vertical[1]);
		}

		let user_items: Vec<ListItem> = self.users()
			.iter()
			.map(|username| ListItem::new(username.as_str()))
			.collect();

		let users = List::new(user_items)
			.block(
				Block::default()
					.title("Users")
					.borders(Borders::ALL),
			);

		frame.render_widget(users, horizontal[1]);
	}

    
}
