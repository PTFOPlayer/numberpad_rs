use lazy_static::lazy_static;
use std::{
    fs::read_to_string,
    process::Command,
    thread::sleep,
    time::{self, Duration, SystemTime},
};

use evdev::{Device, Key as evKey};
use uinput::{self, event::keyboard::Key};

const LEFT_X_OFFSET: usize = 200;
const TOP_Y_OFFSET: usize = 200;
const RIGHT_X_OFFSET: usize = 200;
const BOTTOM_Y_OFFSET: usize = 80;
const MAX_X: usize = 3900;
const MAX_Y: usize = 2300;

const COLS: usize = 5;
const ROWS: usize = 4;
const COL_WIDTH: usize = (MAX_X - RIGHT_X_OFFSET - LEFT_X_OFFSET) / COLS;
const COL_HEIGTH: usize = (MAX_Y - TOP_Y_OFFSET - BOTTOM_Y_OFFSET) / ROWS;

struct KeyWrapper(bool, Key);

lazy_static! {
    static ref KEYS: [[KeyWrapper; 5]; 4] = [
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

// manual commands for dv_ind=1
// sudo i2ctransfer -f -y 1 w13@0x15 0x05 0x00 0x3d 0x03 0x06 0x00 0x07 0x00 0x0d 0x14 0x03 0x60 0xad
// sudo i2ctransfer -f -y 1 w13@0x15 0x05 0x00 0x3d 0x03 0x06 0x00 0x07 0x00 0x0d 0x14 0x03 0x01 0xad
// sudo i2ctransfer -f -y 1 w13@0x15 0x05 0x00 0x3d 0x03 0x06 0x00 0x07 0x00 0x0d 0x14 0x03 0x48 0xad
// sudo i2ctransfer -f -y 1 w13@0x15 0x05 0x00 0x3d 0x03 0x06 0x00 0x07 0x00 0x0d 0x14 0x03 0x00 0xad

#[inline(always)]
fn execute_i2c_transfer(dv_ind: &str, value: u16) {
    let mut cmd = Command::new("sudo");
    cmd.args(&[
        "i2ctransfer",
        "-f",
        "-y",
        dv_ind,
        "w13@0x15",
        "0x05",
        "0x00",
        "0x3d",
        "0x03",
        "0x06",
        "0x00",
        "0x07",
        "0x00",
        "0x0d",
        "0x14",
        "0x03",
        &format!("{:x}", value),
        "0xad",
    ]);

    let out = cmd.output().expect("cannot execute i2ctransfer");
    dbg_t(format!(
        "{}#{}#{}#cmd:{:0x}#dv:{}",
        out.status,
        String::from_utf8(out.stderr).unwrap(),
        String::from_utf8(out.stdout).unwrap(),
        value,
        dv_ind
    ));
}

fn main() {
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

    let i2c = sysfs_split[0]
        .strip_prefix("i2c-")
        .expect("couldn't parse i2c");

    let handlers = devices[section + 4]
        .strip_prefix("H: Handlers=")
        .expect("couldn't find handlers section")
        .split(" ")
        .collect::<Vec<&str>>();

    dbg_t(format!("i2c={}, handlers={:?}", i2c, handlers));

    let mut dur = 1.0;

    let mut state = false;
    let mut hold = false;
    let mut state_inc = 0.0;
    let mut btn = false;

    let event = handlers.into_iter().find(|x| x.contains("event")).unwrap();

    let dev = Device::open("/dev/input/".to_owned() + event).expect("couldn't find handle");

    let mut udev = uinput::default()
        .expect("uinput fail")
        .name("numberpad")
        .unwrap()
        .event(uinput::event::Keyboard::All)
        .expect("uinput fail")
        .create()
        .expect("uinput fail");

    execute_i2c_transfer(i2c, 0x60);
    sleep(Duration::from_secs_f32(0.1));
    execute_i2c_transfer(i2c, 0x01);
    sleep(Duration::from_secs_f32(0.1));
    execute_i2c_transfer(i2c, 0x48);
    sleep(Duration::from_secs_f32(0.1));
    execute_i2c_transfer(i2c, 0x00);
    sleep(Duration::from_secs_f32(0.1));
    loop {
        let press_state = dev.get_key_state().expect("couldn't get device keys");
        let input_state = dev.get_abs_state().expect("couldn't get device state");
        let state_x = input_state[0].value as usize;
        let state_y = input_state[1].value as usize;
        if press_state.contains(evKey::BTN_TOUCH) && state_y < 250 && state_x > MAX_X - 250 && !hold
        {
            state_inc += 1.0 * dur;
            btn = true;
        } else {
            state_inc = 0.0;
            btn = false;
        }

        if !state && state_inc >= 1.9 && !hold {
            execute_i2c_transfer(i2c, 0x60);
            execute_i2c_transfer(i2c, 0x01);
            execute_i2c_transfer(i2c, 0x48);
            state = true;
            state_inc = 0.0;
            dur = 0.10
        } else if state && state_inc >= 1.9 && !hold {
            execute_i2c_transfer(i2c, 0x00);
            state = false;
            state_inc = 0.0;
            dur = 1.0
        }

        if state_x > LEFT_X_OFFSET
            && state_x < MAX_X - RIGHT_X_OFFSET
            && state_y > TOP_Y_OFFSET
            && state_y < MAX_Y - BOTTOM_Y_OFFSET
            && state
            && press_state.contains(evKey::BTN_TOUCH)
            && !hold
        {
            let key_x = (state_x - LEFT_X_OFFSET) / COL_WIDTH;
            let key_y = (state_y - TOP_Y_OFFSET) / COL_HEIGTH;
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

        if press_state.contains(evKey::BTN_TOUCH) && !btn {
            hold = true;
        } else {
            hold = false;
        }

        sleep(Duration::from_secs_f64(dur));
    }
}

fn dbg_t(s: String) {
    let t = time::SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    println!("[{:?}]::{}", t, s);
}
