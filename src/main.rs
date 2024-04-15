use std::{fs::read_to_string, process::exit, thread::sleep, time::Duration};

use evdev::{Device, Key as evKey};
use uinput::{self, event::keyboard::Key};

mod dbg;
use dbg::dbg_t;
mod consts;
use consts::*;
mod i2c;
use i2c::*;
// manual commands for dv_ind=1
// sudo i2ctransfer -f -y 1 w13@0x15 0x05 0x00 0x3d 0x03 0x06 0x00 0x07 0x00 0x0d 0x14 0x03 0x60 0xad
// sudo i2ctransfer -f -y 1 w13@0x15 0x05 0x00 0x3d 0x03 0x06 0x00 0x07 0x00 0x0d 0x14 0x03 0x01 0xad
// sudo i2ctransfer -f -y 1 w13@0x15 0x05 0x00 0x3d 0x03 0x06 0x00 0x07 0x00 0x0d 0x14 0x03 0x48 0xad
// sudo i2ctransfer -f -y 1 w13@0x15 0x05 0x00 0x3d 0x03 0x06 0x00 0x07 0x00 0x0d 0x14 0x03 0x00 0xad

fn main() {
    if sudo::check() != sudo::RunningAs::Root {
        match sudo::escalate_if_needed() {
            Ok(_) => {}
            Err(_) => {
                println!("Root required");
                exit(1);
            }
        }
    }

    let file = read_to_string("/proc/bus/input/devices").expect("msg");

    let splitted = file.split('\n');
    let devices: Vec<&str> = splitted.collect();

    let mut line = 0usize;
    let section = 'l: loop {
        if devices[line].contains("ASUE") && devices[line].contains("Touchpad") {
            break 'l line;
        };
        line += 1;
    };

    let sysfs_split = devices[section + 2]
        .split('/')
        .filter(|x| x.contains("i2c-"))
        .collect::<Vec<&str>>();

    let mut i2c = I2C::new(
        sysfs_split[0]
            .strip_prefix("i2c-")
            .expect("couldn't parse i2c")
            .into(),
    );

    let handlers = devices[section + 4]
        .strip_prefix("H: Handlers=")
        .expect("couldn't find handlers section")
        .split(" ")
        .collect::<Vec<&str>>();

    dbg_t(format!("i2c={:?}, handlers={:?}", i2c, handlers));

    let event = handlers.into_iter().find(|x| x.contains("event")).unwrap();

    let dev_path = "/dev/input/".to_owned() + event;
    dbg_t(format!("device path:{}", dev_path));
    let dev = Device::open(dev_path).expect("couldn't find handle");

    let udev = uinput::default()
        .expect("uinput fail")
        .name("numberpad")
        .unwrap()
        .event(uinput::event::Keyboard::All)
        .expect("uinput fail")
        .create()
        .expect("uinput fail");

    i2c.init();

    dbg_t("Initialization finished".to_owned());

    drive_loop(dev, udev, i2c);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TouchType {
    Hold,
    Normal,
}

impl TouchType {
    pub fn is_hold(&self) -> bool {
        self == &Self::Hold
    }

    pub fn is_normal(&self) -> bool {
        self == &Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum State {
    Normal(TouchType),
    LeftBtn(TouchType),
    RightBtn(TouchType),
}

impl State {
    pub fn get_touch_type(&mut self) -> &mut TouchType {
        match self {
            State::Normal(touch_type) => touch_type,
            State::LeftBtn(touch_type) => touch_type,
            State::RightBtn(touch_type) => touch_type,
        }
    }

    pub fn is_non_btn(&self) -> bool {
        match self {
            State::Normal(_) => true,
            _ => false,
        }
    }

    pub fn set_inner(&mut self, other_touch_type: TouchType) {
        match self {
            State::Normal(touch_type) => *touch_type = other_touch_type,
            State::LeftBtn(touch_type) => *touch_type = other_touch_type,
            State::RightBtn(touch_type) => *touch_type = other_touch_type,
        }
    }
}

struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn check_rbtn(&self) -> bool {
        self.y < 250 && self.x > MAX_X - 250
    }

    fn check_lbtn(&self) -> bool {
        self.y < 250 && self.x < 250
    }

    fn in_bounds(&self) -> bool {
        self.x > LEFT_X_OFFSET
            && self.x < MAX_X - RIGHT_X_OFFSET
            && self.y > TOP_Y_OFFSET
            && self.y < MAX_Y - BOTTOM_Y_OFFSET
    }
}

fn drive_loop(dev: Device, mut udev: uinput::Device, mut i2c: I2C) {
    let mut state = State::Normal(TouchType::Normal);
    let mut is_on = false;

    let mut state_inc = 0.0;
    let mut dur = 1.0;

    loop {
        let press_state = dev.get_key_state().expect("couldn't get device keys");
        let input = dev.get_abs_state().expect("couldn't get device state");
        let position = Position::new(input[0].value as usize, input[1].value as usize);

        if press_state.contains(evKey::BTN_TOUCH)
            && state.get_touch_type().is_normal()
            && position.check_rbtn()
        {
            state = State::RightBtn(TouchType::Normal);

            state_inc += 1.0 * dur;
        } else if press_state.contains(evKey::BTN_TOUCH)
            && state.get_touch_type().is_normal()
            && position.check_lbtn()
        {
            state = State::LeftBtn(TouchType::Normal);

            state_inc += 1.0 * dur;
        } else {
            state = State::Normal(TouchType::Normal);
            state_inc = 0.0;
        }

        if !is_on && state_inc >= 1.9 && state.get_touch_type().is_normal() {
            i2c.on();
            is_on = true;
            state_inc = 0.0;
            dur = 0.10
        } else if is_on && state_inc >= 1.9 && state.get_touch_type().is_normal() {
            i2c.off();
            is_on = false;
            state_inc = 0.0;
            dur = 1.0
        }

        if position.in_bounds()
            && is_on
            && press_state.contains(evKey::BTN_TOUCH)
            && !state.get_touch_type().is_hold()
        {
            let key_x = (position.x - LEFT_X_OFFSET) / COL_WIDTH;
            let key_y = (position.y - TOP_Y_OFFSET) / COL_HEIGTH;
            if key_x <= 4 && key_y <= 3 {
                if !KEYS[key_y][key_x].0 {
                    udev.click(&KEYS[key_y][key_x].1).expect("key error");
                } else {
                    udev.press(&Key::LeftShift).expect("key error");
                    udev.click(&KEYS[key_y][key_x].1).expect("key error");
                    udev.release(&Key::LeftShift).expect("key error");
                }

                let _ = udev.synchronize();
            }
        }

        if press_state.contains(evKey::BTN_TOUCH) && state.is_non_btn() {
            state.set_inner(TouchType::Hold)
        } else {
            state.set_inner(TouchType::Normal)
        }

        sleep(Duration::from_secs_f64(dur));
    }
}
