#![no_std]
#![no_main]

extern crate panic_halt;

use riscv_rt::entry;
use hifive1::hal::i2c::{I2c, Speed};
use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};

#[entry]
fn main() -> ! {
    let dr = DeviceResources::take().unwrap();
    let p = dr.peripherals;
    let pins = dr.pins;

    // Configure clocks
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 320.mhz().into());

    // Configure UART for stdout
    hifive1::stdout::configure(
        p.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );

    sprintln!("hIMUdev app v0.1.0");

    // Configure I2C
    let sda = pin!(pins, i2c0_sda).into_iof0();
    let scl = pin!(pins, i2c0_scl).into_iof0();
    let mut i2c = I2c::new(p.I2C0, sda, scl, Speed::Normal, clocks);

    // Read ID from BMP180 sensor (register 0xD0)
    let mut send_buffer = [0xd0];
    let mut recv_buffer = [0u8; 0x1];
    match i2c.write_read(0x77, &send_buffer, &mut recv_buffer) {
        Ok(_) => sprintln!("Data received = {:?}", recv_buffer),
        Err(e) => sprintln!("Error: {:?}", e),
    }

    // Read calibration data from BMP180 sensor (registers 0xAA..0xBF)
    send_buffer = [0xaa];
    let mut recv_buffer = [0u8; 0x15];
    match i2c.write_read(0x77, &send_buffer, &mut recv_buffer) {
        Ok(_) => sprintln!("Data received = {:?}", recv_buffer),
        Err(e) => sprintln!("Error: {:?}", e),
    }

    loop {}
}
