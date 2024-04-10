use std::{process::Command, rc::Rc};

use crate::dbg::dbg_t;

#[derive(Debug)]
pub struct I2C {
    id: Rc<str>,
}

impl I2C {
    pub fn new(id: Rc<str>) -> Self {
        Self { id }
    }

    pub fn on(&self) {
        self.execute_i2c_transfer("0x60");
        self.execute_i2c_transfer("0x48");
        self.execute_i2c_transfer("0x01");
        self.execute_i2c_transfer("0x48");
    }

    pub fn off(&self) {
        self.execute_i2c_transfer("0x00");
        self.execute_i2c_transfer("0x61");
    }

    pub fn init(&self) {
        self.execute_i2c_transfer("0x60");
        self.execute_i2c_transfer("0x48");
        self.execute_i2c_transfer("0x01");
        self.execute_i2c_transfer("0x48");
        self.execute_i2c_transfer("0x00");
    }

    #[inline(always)]
    fn execute_i2c_transfer(&self, value: &str) {
        let mut cmd = Command::new("sudo");
        cmd.args(&[
            "i2ctransfer",
            "-f",
            "-y",
            &self.id,
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
            value,
            "0xad",
        ]);

        let out = cmd.output().expect("cannot execute i2ctransfer");
        dbg_t(format!(
            "{}#{}#{}#cmd:{}#dv:{}",
            out.status,
            String::from_utf8(out.stderr).unwrap(),
            String::from_utf8(out.stdout).unwrap(),
            value,
            self.id
        ));
    }
}
