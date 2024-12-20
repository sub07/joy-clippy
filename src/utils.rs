use iced::{keyboard::key, Color, Event};

use crate::app::Shortcut;

pub const ASYNC_CHANNEL_SIZE: usize = 10;

pub trait ColorUtils {
    fn darken(self, percentage: f32) -> Self;
    fn lighten(self, percentage: f32) -> Self;
}

impl ColorUtils for Color {
    fn darken(self, percentage: f32) -> Self {
        let Color { r, g, b, a } = self;
        let factor = 1.0 - percentage;
        Color::from_rgba(
            (r * factor).clamp(0.0, 1.0),
            (g * factor).clamp(0.0, 1.0),
            (b * factor).clamp(0.0, 1.0),
            a,
        )
    }

    fn lighten(self, percentage: f32) -> Self {
        let Color { r, g, b, a } = self;
        let factor = 1.0 + percentage;
        Color::from_rgba(
            (r * factor).clamp(0.0, 1.0),
            (g * factor).clamp(0.0, 1.0),
            (b * factor).clamp(0.0, 1.0),
            a,
        )
    }
}

pub fn iced_key_to_rdev(key: iced::keyboard::key::Physical) -> rdev::Key {
    match key {
        key::Physical::Code(code) => match code {
            key::Code::Backquote => rdev::Key::BackQuote,
            key::Code::Backslash => rdev::Key::BackSlash,
            key::Code::BracketLeft => rdev::Key::LeftBracket,
            key::Code::BracketRight => rdev::Key::RightBracket,
            key::Code::Comma => rdev::Key::Comma,
            key::Code::Digit0 => rdev::Key::Num0,
            key::Code::Digit1 => rdev::Key::Num1,
            key::Code::Digit2 => rdev::Key::Num2,
            key::Code::Digit3 => rdev::Key::Num3,
            key::Code::Digit4 => rdev::Key::Num4,
            key::Code::Digit5 => rdev::Key::Num5,
            key::Code::Digit6 => rdev::Key::Num6,
            key::Code::Digit7 => rdev::Key::Num7,
            key::Code::Digit8 => rdev::Key::Num8,
            key::Code::Digit9 => rdev::Key::Num9,
            key::Code::Equal => rdev::Key::Equal,
            key::Code::IntlBackslash => rdev::Key::IntlBackslash,
            key::Code::KeyA => rdev::Key::KeyA,
            key::Code::KeyB => rdev::Key::KeyB,
            key::Code::KeyC => rdev::Key::KeyC,
            key::Code::KeyD => rdev::Key::KeyD,
            key::Code::KeyE => rdev::Key::KeyE,
            key::Code::KeyF => rdev::Key::KeyF,
            key::Code::KeyG => rdev::Key::KeyG,
            key::Code::KeyH => rdev::Key::KeyH,
            key::Code::KeyI => rdev::Key::KeyI,
            key::Code::KeyJ => rdev::Key::KeyJ,
            key::Code::KeyK => rdev::Key::KeyK,
            key::Code::KeyL => rdev::Key::KeyL,
            key::Code::KeyM => rdev::Key::KeyM,
            key::Code::KeyN => rdev::Key::KeyN,
            key::Code::KeyO => rdev::Key::KeyO,
            key::Code::KeyP => rdev::Key::KeyP,
            key::Code::KeyQ => rdev::Key::KeyQ,
            key::Code::KeyR => rdev::Key::KeyR,
            key::Code::KeyS => rdev::Key::KeyS,
            key::Code::KeyT => rdev::Key::KeyT,
            key::Code::KeyU => rdev::Key::KeyU,
            key::Code::KeyV => rdev::Key::KeyV,
            key::Code::KeyW => rdev::Key::KeyW,
            key::Code::KeyX => rdev::Key::KeyX,
            key::Code::KeyY => rdev::Key::KeyY,
            key::Code::KeyZ => rdev::Key::KeyZ,
            key::Code::Minus => rdev::Key::Minus,
            key::Code::Period => rdev::Key::Dot,
            key::Code::Quote => rdev::Key::Quote,
            key::Code::Semicolon => rdev::Key::SemiColon,
            key::Code::Slash => rdev::Key::Slash,
            key::Code::AltLeft => rdev::Key::Alt,
            key::Code::AltRight => rdev::Key::Alt,
            key::Code::Backspace => rdev::Key::Backspace,
            key::Code::CapsLock => rdev::Key::CapsLock,
            key::Code::ControlLeft => rdev::Key::ControlLeft,
            key::Code::ControlRight => rdev::Key::ControlRight,
            key::Code::Enter => rdev::Key::Return,
            key::Code::SuperLeft => rdev::Key::MetaLeft,
            key::Code::SuperRight => rdev::Key::MetaRight,
            key::Code::ShiftLeft => rdev::Key::ShiftLeft,
            key::Code::ShiftRight => rdev::Key::ShiftRight,
            key::Code::Space => rdev::Key::Space,
            key::Code::Tab => rdev::Key::Tab,
            key::Code::Delete => rdev::Key::Delete,
            key::Code::End => rdev::Key::End,
            key::Code::Home => rdev::Key::Home,
            key::Code::Insert => rdev::Key::Insert,
            key::Code::PageDown => rdev::Key::PageDown,
            key::Code::PageUp => rdev::Key::PageUp,
            key::Code::ArrowDown => rdev::Key::DownArrow,
            key::Code::ArrowLeft => rdev::Key::LeftArrow,
            key::Code::ArrowRight => rdev::Key::RightArrow,
            key::Code::ArrowUp => rdev::Key::UpArrow,
            key::Code::NumLock => rdev::Key::NumLock,
            key::Code::Numpad0 => rdev::Key::Num0,
            key::Code::Numpad1 => rdev::Key::Num1,
            key::Code::Numpad2 => rdev::Key::Num2,
            key::Code::Numpad3 => rdev::Key::Num3,
            key::Code::Numpad4 => rdev::Key::Num4,
            key::Code::Numpad5 => rdev::Key::Num5,
            key::Code::Numpad6 => rdev::Key::Num6,
            key::Code::Numpad7 => rdev::Key::Num7,
            key::Code::Numpad8 => rdev::Key::Num8,
            key::Code::Numpad9 => rdev::Key::Num9,
            key::Code::NumpadAdd => rdev::Key::KpPlus,
            key::Code::NumpadDivide => rdev::Key::KpDivide,
            key::Code::NumpadEnter => rdev::Key::KpReturn,
            key::Code::NumpadMultiply => rdev::Key::KpMultiply,
            key::Code::NumpadStar => rdev::Key::KpMultiply,
            key::Code::NumpadSubtract => rdev::Key::KpMinus,
            key::Code::Escape => rdev::Key::Escape,
            key::Code::Fn => rdev::Key::Function,
            key::Code::PrintScreen => rdev::Key::PrintScreen,
            key::Code::ScrollLock => rdev::Key::ScrollLock,
            key::Code::Pause => rdev::Key::Pause,
            key::Code::Meta => rdev::Key::MetaLeft,
            key::Code::F1 => rdev::Key::F1,
            key::Code::F2 => rdev::Key::F2,
            key::Code::F3 => rdev::Key::F3,
            key::Code::F4 => rdev::Key::F4,
            key::Code::F5 => rdev::Key::F5,
            key::Code::F6 => rdev::Key::F6,
            key::Code::F7 => rdev::Key::F7,
            key::Code::F8 => rdev::Key::F8,
            key::Code::F9 => rdev::Key::F9,
            key::Code::F10 => rdev::Key::F10,
            key::Code::F11 => rdev::Key::F11,
            key::Code::F12 => rdev::Key::F12,
            _ => rdev::Key::Unknown(0),
        },
        key::Physical::Unidentified(native_code) => match native_code {
            key::NativeCode::Windows(code) => rdev::Key::Unknown(code as u32),
            _ => rdev::Key::Unknown(0),
        },
    }
}

pub fn iced_event_to_shortcut(event: iced::Event) -> Option<Shortcut> {
    match event {
        Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key,
            modified_key: _,
            physical_key,
            location: _,
            modifiers,
            text: _,
        }) => Some(Shortcut {
            modifiers,
            logical_key: key,
            iced_physical_key: physical_key,
            rdev_key: iced_key_to_rdev(physical_key),
        }),
        _ => None,
    }
}
