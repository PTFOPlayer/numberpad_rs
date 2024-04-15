use std::{fmt::Debug, rc::Rc};

use i2cdev::{
    core::{I2CMessage, I2CTransfer},
    linux::{LinuxI2CDevice, LinuxI2CMessage},
};

use crate::dbg::dbg_t;

pub struct I2C {
    id: Rc<str>,
    i2c: LinuxI2CDevice,
}

impl I2C {
    pub fn new(id: Rc<str>) -> Self {
        let path = format!("/dev/i2c-{}", id);
        Self {
            id,
            i2c: unsafe { LinuxI2CDevice::force_new(path, 0x15).unwrap() },
        }
    }

    pub fn on(&mut self) {
        self.execute_i2c_transfer(0x60);
        self.execute_i2c_transfer(0x48);
        self.execute_i2c_transfer(0x01);
        self.execute_i2c_transfer(0x48);
    }

    pub fn off(&mut self) {
        self.execute_i2c_transfer(0x00);
        self.execute_i2c_transfer(0x61);
    }

    pub fn init(&mut self) {
        self.execute_i2c_transfer(0x60);
        self.execute_i2c_transfer(0x48);
        self.execute_i2c_transfer(0x01);
        self.execute_i2c_transfer(0x48);
        self.execute_i2c_transfer(0x00);
        self.execute_i2c_transfer(0x61);
    }

    #[inline(always)]
    fn execute_i2c_transfer(&mut self, value: u8) {
        let binding = [
            0x05, 0x00, 0x3d, 0x03, 0x06, 0x00, 0x07, 0x00, 0x0d, 0x14, 0x03, value, 0xad,
        ];
        let mut msg = [LinuxI2CMessage::write(&binding)];
        match self.i2c.transfer(&mut msg) {
            Ok(_) => dbg_t(format!("id:{}, value:{:2x}", self.id, value)),
            Err(err) => dbg_t(format!("err:{}", err)),
        };
    }
}

impl Debug for I2C {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I2C").field("id", &self.id).finish()
    }
}
