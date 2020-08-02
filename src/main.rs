#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::delay::Sleep;
use hifive1::hal::prelude::*;
use hifive1::hal::spi::{Spi, MODE_0};
use hifive1::hal::DeviceResources;
use hifive1::{pin, pins, sprintln, Led};
use riscv_rt::entry;

// switches led according to supplied status returning the new state back
fn toggle_led(led: &mut dyn Led, status: bool) -> bool {
    match status {
        true => led.on(),
        false => led.off(),
    }

    !status
}

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

    // Configure SPI pins
    let mosi = pin!(pins, spi0_mosi).into_iof0();
    let miso = pin!(pins, spi0_miso).into_iof0();
    let sck = pin!(pins, spi0_sck).into_iof0();
    let cs = pin!(pins, spi0_ss0).into_iof0();

    // Configure SPI
    let spi_pins = (mosi, miso, sck, cs);
    let mut spi = Spi::new(p.QSPI1, spi_pins, MODE_0, 1_000_000.hz(), clocks);

    let mut buf = [0x80, 0x00];
    let _ = spi.transfer(&mut buf);

    sprintln!("{:?}", buf);

    // get all 3 led pins in a tuple (each pin is it's own type here)
    let led_pins = pins!(pins, (led_red, led_green, led_blue, led_yellow));
    let mut tleds = hifive1::all(led_pins.0, led_pins.1, led_pins.2, led_pins.3);

    // get leds as the Led trait in an array so we can index them
    let ileds: [&mut dyn Led; 4] = [&mut tleds.0, &mut tleds.1, &mut tleds.2, &mut tleds.3];

    // get the local interrupts struct
    let clint = dr.core_peripherals.clint;

    let mut led_status = [true, true, true, true]; // start on red
    let mut current_led = 0; // start on red

    // get the sleep struct
    let mut sleep = Sleep::new(clint.mtimecmp, clocks);

    const PERIOD: u32 = 1000; // 1s
    loop {
        // toggle led
        led_status[current_led] = toggle_led(ileds[current_led], led_status[current_led]);

        // increment index if we blinked back to blank
        if led_status[current_led] {
            current_led = (current_led + 1) % 4
        }

        let mut buf = [0x80, 0x00];
        let _ = spi.transfer(&mut buf);

        sprintln!("{:?}", buf);

        // sleep for 1
        sleep.delay_ms(PERIOD);
    }
}
