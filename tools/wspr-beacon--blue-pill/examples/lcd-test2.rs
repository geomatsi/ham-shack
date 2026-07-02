#![no_main]
#![no_std]

use bitbang_hal::i2c::I2cBB;
use cortex_m_rt::entry;
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use hal::prelude::*;
use nb::block;
use panic_rtt_target as _;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};
use stm32f1xx_hal as hal;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::rcc;
use wspr_beacon::support::bitbang_i2c_compat::Eh1BitBangI2c;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.freeze(
        rcc::Config::hse(8.MHz()).sysclk(32.MHz()).pclk1(16.MHz()),
        &mut flash.acr,
    );

    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut tmr = dp.TIM3.counter_hz(&mut rcc);

    let scl = gpioa.pa0.into_open_drain_output(&mut gpioa.crl);
    let sda = gpioa.pa1.into_open_drain_output(&mut gpioa.crl);
    let mut i2c_timer = dp.TIM2.counter_hz(&mut rcc);
    i2c_timer.start(100.kHz()).unwrap();
    let i2c = I2cBB::new(scl, sda, i2c_timer);
    let i2c = Eh1BitBangI2c::new(i2c);

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    tmr.start(5.Hz()).unwrap();

    loop {
        for c in 0..10 {
            let _ = display.clear(BinaryColor::Off);
            Text::with_baseline(
                "Hello World !",
                Point::new(c * 3, c * 3),
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)
            .unwrap();
            display.flush().unwrap();
            block!(tmr.wait()).ok();
        }
    }
}
