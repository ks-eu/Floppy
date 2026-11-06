use crossterm::event::{
    Event,
    KeyCode,
    KeyEventKind,
};

pub enum InputEvent {
    KeyStroke(char),
    BackSpace,
    Enter,
    Exit,
    Error
}

pub fn keypress(event: Event) -> Option<InputEvent> {
    let Event::Key(key) = event else {
        return None;
    };

    if key.kind != KeyEventKind::Press {
        return None;
    }

    match key.code {
        KeyCode::Char(c) => Some(InputEvent::KeyStroke(c)),
        KeyCode::Backspace => Some(InputEvent::BackSpace),
        KeyCode::Enter => Some(InputEvent::Enter),
        KeyCode::Esc => Some(InputEvent::Exit),
        _ => None,
    }
}
