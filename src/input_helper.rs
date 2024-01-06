use macroquad::{
    input::{get_last_key_pressed, is_key_down},
    miniquad::KeyCode,
};

pub enum InputAction {
    NumberEntered(u8),
    Function(u8),
    Reset,
    Clear,
    AutoPlay,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

impl TryFrom<KeyCode> for InputAction {
    type Error = String;
    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        Ok(match value {
            KeyCode::Key1 => InputAction::NumberEntered(1),
            KeyCode::Key2 => InputAction::NumberEntered(2),
            KeyCode::Key3 => InputAction::NumberEntered(3),
            KeyCode::Key4 => InputAction::NumberEntered(4),
            KeyCode::Key5 => InputAction::NumberEntered(5),
            KeyCode::Key6 => InputAction::NumberEntered(6),
            KeyCode::Key7 => InputAction::NumberEntered(7),
            KeyCode::Key8 => InputAction::NumberEntered(8),
            KeyCode::Key9 => InputAction::NumberEntered(9),
            KeyCode::Backspace | KeyCode::Key0 | KeyCode::Delete => InputAction::Clear,
            KeyCode::Space => InputAction::AutoPlay,
            KeyCode::Tab => InputAction::Reset,
            KeyCode::F1 => InputAction::Function(1),
            KeyCode::F2 => InputAction::Function(2),
            KeyCode::F3 => InputAction::Function(3),
            KeyCode::F4 => InputAction::Function(4),
            KeyCode::F5 => InputAction::Function(5),
            KeyCode::F6 => InputAction::Function(6),
            KeyCode::F7 => InputAction::Function(7),
            KeyCode::F8 => InputAction::Function(8),
            KeyCode::F9 => InputAction::Function(9),
            KeyCode::F10 => InputAction::Function(10),
            KeyCode::F11 => InputAction::Function(11),
            KeyCode::F12 => InputAction::Function(12),
            KeyCode::W | KeyCode::Up => InputAction::MoveUp,
            KeyCode::A | KeyCode::Left => InputAction::MoveLeft,
            KeyCode::S | KeyCode::Down => InputAction::MoveDown,
            KeyCode::D | KeyCode::Right => InputAction::MoveRight,
            _ => Err("Not a recognised key".to_string())?,
        })
    }
}

impl InputAction {
    pub fn get_last_input() -> Option<InputAction> {
        get_last_key_pressed().and_then(|key| InputAction::try_from(key).ok())
    }

    pub fn is_function_down(num: u8) -> bool {
        match num {
            1 => is_key_down(KeyCode::F1),
            2 => is_key_down(KeyCode::F2),
            3 => is_key_down(KeyCode::F3),
            4 => is_key_down(KeyCode::F4),
            5 => is_key_down(KeyCode::F5),
            6 => is_key_down(KeyCode::F6),
            7 => is_key_down(KeyCode::F7),
            8 => is_key_down(KeyCode::F8),
            9 => is_key_down(KeyCode::F9),
            10 => is_key_down(KeyCode::F10),
            11 => is_key_down(KeyCode::F11),
            12 => is_key_down(KeyCode::F12),
            _ => false,
        }
    }
}
