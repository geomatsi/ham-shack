#![no_main]
#![no_std]

#[cfg(feature = "rtt-log")]
use panic_rtt_target as _;

#[cfg(feature = "rtt-log")]
use rtt_target::{rprintln, rtt_init_print};

#[cfg(not(feature = "rtt-log"))]
use panic_halt as _;

use bitbang_hal::i2c::I2cBB;
use core::fmt::Write;
use cortex_m_rt::entry;
use hal::prelude::*;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};
use stm32f1xx_hal as hal;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::rcc;
use wspr_beacon::support::bitbang_i2c_compat::Eh1BitBangI2c;
use wspr_beacon::wspr_log;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.freeze(
        rcc::Config::hse(8.MHz()).sysclk(32.MHz()).pclk1(16.MHz()),
        &mut flash.acr,
    );

    #[cfg(feature = "rtt-log")]
    rtt_init_print!();

    let mut gpioa = dp.GPIOA.split(&mut rcc);

    let scl = gpioa.pa0.into_open_drain_output(&mut gpioa.crl);
    let sda = gpioa.pa1.into_open_drain_output(&mut gpioa.crl);
    let mut i2c_timer = dp.TIM2.counter_hz(&mut rcc);
    i2c_timer.start(100.kHz()).unwrap();
    let i2c = I2cBB::new(scl, sda, i2c_timer);
    let i2c = Eh1BitBangI2c::new(i2c);

    let interface = I2CDisplayInterface::new(i2c);
    let mut display =
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_terminal_mode();
    display.init().unwrap();
    let _ = display.clear();

    /* Endless loop */
    loop {
        wspr_log!("test #1");

        for c in 97..123 {
            let buf = [c];
            let s = core::str::from_utf8(&buf).unwrap();
            let _ = display.write_str(s);
        }

        wspr_log!("test #2");

        for c in 65..91 {
            let buf = [c];
            let s = core::str::from_utf8(&buf).unwrap();
            let _ = display.write_str(s);
        }
    }
}
