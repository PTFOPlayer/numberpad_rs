use lazy_static::lazy_static;
use uinput::event::keyboard::Key;

pub const LEFT_X_OFFSET: usize = 200;
pub const TOP_Y_OFFSET: usize = 200;
pub const RIGHT_X_OFFSET: usize = 200;
pub const BOTTOM_Y_OFFSET: usize = 80;
pub const MAX_X: usize = 3900;
pub const MAX_Y: usize = 2300;

pub const COLS: usize = 5;
pub const ROWS: usize = 4;
pub const COL_WIDTH: usize = (MAX_X - RIGHT_X_OFFSET - LEFT_X_OFFSET) / COLS;
pub const COL_HEIGTH: usize = (MAX_Y - TOP_Y_OFFSET - BOTTOM_Y_OFFSET) / ROWS;

pub struct KeyWrapper(pub bool, pub Key);

lazy_static! {
    pub static ref KEYS: [[KeyWrapper; 5]; 4] = [
        [
            KeyWrapper(false, Key::_7),
            KeyWrapper(false, Key::_8),
            KeyWrapper(false, Key::_9),
            KeyWrapper(false, Key::Slash),
            KeyWrapper(false, Key::BackSpace),
        ],
        [
            KeyWrapper(false, Key::_4),
            KeyWrapper(false, Key::_5),
            KeyWrapper(false, Key::_6),
            KeyWrapper(true, Key::_8),
            KeyWrapper(false, Key::BackSpace),
        ],
        [
            KeyWrapper(false, Key::_1),
            KeyWrapper(false, Key::_2),
            KeyWrapper(false, Key::_3),
            KeyWrapper(false, Key::Minus),
            KeyWrapper(true, Key::_5),
        ],
        [
            KeyWrapper(false, Key::_0),
            KeyWrapper(false, Key::Dot),
            KeyWrapper(false, Key::Enter),
            KeyWrapper(true, Key::Equal),
            KeyWrapper(false, Key::Equal),
        ],
    ];
}
