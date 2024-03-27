use macroquad::{
    input::{get_last_key_pressed, is_key_down},
    miniquad::KeyCode,
};

#[derive(Clone, Copy)]
pub enum InputActionContext {
    Buffer,
    Generic,
}

pub enum InputActionChar {
    Char(char),
    Backspace,
    Clear,
}

#[derive(Debug)]
pub enum InputAction {
    NumberEntered(u8),
    Function(u8),
    Reset,
    HardReset,
    Clear,
    AutoPlay,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    ClearBuffer,
    PasteBuffer,
    EnterBuffer,
    UpBuffer,
    DownBuffer,
}

pub const TYPE_BUFFER_KEY: KeyCode = KeyCode::LeftControl;

#[derive(Default)]
pub struct InputState {
    pub enter_buffer: bool,
}

impl InputAction {
    fn try_from(value: KeyCode, state: &InputState) -> Result<Self, String> {
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
            KeyCode::Tab => {
                if is_key_down(KeyCode::LeftShift) {
                    InputAction::HardReset
                } else {
                    InputAction::Reset
                }
            }
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
            KeyCode::W => InputAction::MoveUp,
            KeyCode::A | KeyCode::Left => InputAction::MoveLeft,
            KeyCode::S => InputAction::MoveDown,
            KeyCode::D | KeyCode::Right => InputAction::MoveRight,
            KeyCode::Up => {
                if state.enter_buffer {
                    InputAction::UpBuffer
                } else {
                    InputAction::MoveUp
                }
            }
            KeyCode::Down => {
                if state.enter_buffer {
                    InputAction::DownBuffer
                } else {
                    InputAction::MoveDown
                }
            }
            TYPE_BUFFER_KEY => InputAction::ClearBuffer,
            KeyCode::V => {
                if state.enter_buffer && is_key_down(KeyCode::LeftAlt) {
                    InputAction::PasteBuffer
                } else {
                    Err("Not a recognised key".to_string())?
                }
            }
            KeyCode::Enter => InputAction::EnterBuffer,
            _ => Err("Not a recognised key".to_string())?,
        })
    }

    pub fn get_last_key_pressed(ctx: InputActionContext, state: &InputState) -> Option<KeyCode> {
        let key = get_last_key_pressed();
        let typing_buffer = state.enter_buffer;

        match (typing_buffer, ctx) {
            (false, InputActionContext::Generic) | (true, InputActionContext::Buffer) => key,
            _ => None,
        }
    }

    pub fn is_key_down(key: KeyCode, ctx: InputActionContext, state: &InputState) -> bool {
        let key = is_key_down(key);
        let typing_buffer = state.enter_buffer;

        match (typing_buffer, ctx) {
            (false, InputActionContext::Generic) | (true, InputActionContext::Buffer) => key,
            _ => false,
        }
    }

    pub fn get_last_input(ctx: InputActionContext, state: &InputState) -> Option<InputAction> {
        Self::get_last_key_pressed(ctx, state)
            .and_then(|key| InputAction::try_from(key, state).ok())
    }

    pub fn is_function_pressed(num: u8, ctx: InputActionContext, state: &InputState) -> bool {
        let last_key_pressed = Self::get_last_input(ctx, state);
        if let Some(InputAction::Function(i)) = last_key_pressed {
            return i == num;
        }
        false
    }

    pub fn is_function_down(num: u8, ctx: InputActionContext, state: &InputState) -> bool {
        match num {
            1 => Self::is_key_down(KeyCode::F1, ctx, state),
            2 => Self::is_key_down(KeyCode::F2, ctx, state),
            3 => Self::is_key_down(KeyCode::F3, ctx, state),
            4 => Self::is_key_down(KeyCode::F4, ctx, state),
            5 => Self::is_key_down(KeyCode::F5, ctx, state),
            6 => Self::is_key_down(KeyCode::F6, ctx, state),
            7 => Self::is_key_down(KeyCode::F7, ctx, state),
            8 => Self::is_key_down(KeyCode::F8, ctx, state),
            9 => Self::is_key_down(KeyCode::F9, ctx, state),
            10 => Self::is_key_down(KeyCode::F10, ctx, state),
            11 => Self::is_key_down(KeyCode::F11, ctx, state),
            12 => Self::is_key_down(KeyCode::F12, ctx, state),
            _ => false,
        }
    }

    fn to_raw_char(key_code: KeyCode, ctx: InputActionContext, state: &InputState) -> Option<char> {
        let is_shift_pressed = Self::is_key_down(KeyCode::LeftShift, ctx, state);
        match key_code {
            KeyCode::Space => Some(' '),
            KeyCode::Apostrophe => Some(if is_shift_pressed { '"' } else { '\'' }),
            KeyCode::Comma => Some(if is_shift_pressed { '<' } else { ',' }),
            KeyCode::Minus => Some(if is_shift_pressed { '_' } else { '-' }),
            KeyCode::Period => Some(if is_shift_pressed { '>' } else { '.' }),
            KeyCode::Slash => Some(if is_shift_pressed { '?' } else { '/' }),
            KeyCode::Key0 => Some(if is_shift_pressed { ')' } else { '0' }),
            KeyCode::Key1 => Some(if is_shift_pressed { '!' } else { '1' }),
            KeyCode::Key2 => Some(if is_shift_pressed { '@' } else { '2' }),
            KeyCode::Key3 => Some(if is_shift_pressed { '#' } else { '3' }),
            KeyCode::Key4 => Some(if is_shift_pressed { '$' } else { '4' }),
            KeyCode::Key5 => Some(if is_shift_pressed { '%' } else { '5' }),
            KeyCode::Key6 => Some(if is_shift_pressed { '^' } else { '6' }),
            KeyCode::Key7 => Some(if is_shift_pressed { '&' } else { '7' }),
            KeyCode::Key8 => Some(if is_shift_pressed { '*' } else { '8' }),
            KeyCode::Key9 => Some(if is_shift_pressed { '(' } else { '9' }),
            KeyCode::Semicolon => Some(if is_shift_pressed { ':' } else { ';' }),
            KeyCode::Equal => Some(if is_shift_pressed { '+' } else { '=' }),
            KeyCode::A => Some(if is_shift_pressed { 'A' } else { 'a' }),
            KeyCode::B => Some(if is_shift_pressed { 'B' } else { 'b' }),
            KeyCode::C => Some(if is_shift_pressed { 'C' } else { 'c' }),
            KeyCode::D => Some(if is_shift_pressed { 'D' } else { 'd' }),
            KeyCode::E => Some(if is_shift_pressed { 'E' } else { 'e' }),
            KeyCode::F => Some(if is_shift_pressed { 'F' } else { 'f' }),
            KeyCode::G => Some(if is_shift_pressed { 'G' } else { 'g' }),
            KeyCode::H => Some(if is_shift_pressed { 'H' } else { 'h' }),
            KeyCode::I => Some(if is_shift_pressed { 'I' } else { 'i' }),
            KeyCode::J => Some(if is_shift_pressed { 'J' } else { 'j' }),
            KeyCode::K => Some(if is_shift_pressed { 'K' } else { 'k' }),
            KeyCode::L => Some(if is_shift_pressed { 'L' } else { 'l' }),
            KeyCode::M => Some(if is_shift_pressed { 'M' } else { 'm' }),
            KeyCode::N => Some(if is_shift_pressed { 'N' } else { 'n' }),
            KeyCode::O => Some(if is_shift_pressed { 'O' } else { 'o' }),
            KeyCode::P => Some(if is_shift_pressed { 'P' } else { 'p' }),
            KeyCode::Q => Some(if is_shift_pressed { 'Q' } else { 'q' }),
            KeyCode::R => Some(if is_shift_pressed { 'R' } else { 'r' }),
            KeyCode::S => Some(if is_shift_pressed { 'S' } else { 's' }),
            KeyCode::T => Some(if is_shift_pressed { 'T' } else { 't' }),
            KeyCode::U => Some(if is_shift_pressed { 'U' } else { 'u' }),
            KeyCode::V => Some(if is_shift_pressed { 'V' } else { 'v' }),
            KeyCode::W => Some(if is_shift_pressed { 'W' } else { 'w' }),
            KeyCode::X => Some(if is_shift_pressed { 'X' } else { 'x' }),
            KeyCode::Y => Some(if is_shift_pressed { 'Y' } else { 'y' }),
            KeyCode::Z => Some(if is_shift_pressed { 'Z' } else { 'z' }),
            KeyCode::LeftBracket => Some(if is_shift_pressed { '{' } else { '[' }),
            KeyCode::Backslash => Some(if is_shift_pressed { '|' } else { '\\' }),
            KeyCode::RightBracket => Some(if is_shift_pressed { '}' } else { ']' }),
            KeyCode::GraveAccent => Some(if is_shift_pressed { '~' } else { '`' }),
            // Handle other cases as needed
            _ => None,
        }
    }

    pub fn get_last_input_char(
        ctx: InputActionContext,
        state: &InputState,
    ) -> Option<InputActionChar> {
        Self::get_last_key_pressed(ctx, state).and_then(|x| match x {
            KeyCode::Backspace => Some(InputActionChar::Backspace),
            KeyCode::Escape => Some(InputActionChar::Clear),
            x => Self::to_raw_char(x, ctx, state).map(InputActionChar::Char),
        })
    }
}
