#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_rt::entry;
use panic_semihosting as _;
use si5351::{ClockOutput, Frequency, PLL, Si5351, Si5351Device, DriveStrength};
use stm32f1xx_hal::gpio::{ErasedPin, Output, PushPull};
use stm32f1xx_hal::pac;
use stm32f1xx_hal::rcc;
use stm32f1xx_hal::{
    i2c::{DutyCycle, Mode},
    prelude::*,
};

use wspr_beacon::beacon::config;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cm::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.freeze(
        rcc::Config::hse(8.MHz()).sysclk(32.MHz()).pclk1(16.MHz()),
        &mut flash.acr,
    );

    let mut afio = dp.AFIO.constrain(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);
    let mut delay = cp.SYST.delay(&rcc.clocks);

    let mut led: ErasedPin<Output<PushPull>> = match config::CFG.hw.model {
        config::BluePill::Classic => {
            let mut gpioc = dp.GPIOC.split(&mut rcc);
            gpioc.pc13.into_push_pull_output(&mut gpioc.crh).erase()
        }
        config::BluePill::Plus => gpiob.pb2.into_push_pull_output(&mut gpiob.crl).erase(),
    };

    let scl = gpiob.pb8;
    let sda = gpiob.pb9;

    let i2c = dp.I2C1.remap(&mut afio.mapr).blocking_i2c(
        (scl, sda),
        Mode::Fast {
            frequency: 400.kHz(),
            duty_cycle: DutyCycle::Ratio2to1,
        },
        &mut rcc,
        1000,
        10,
        1000,
        1000,
    );

    let mut clock = Si5351Device::new_adafruit_module(i2c);
    clock.init_adafruit_module().unwrap();

    let freq = 7_000_000u32; //14_000_000u32;

    clock
        .set_frequency(PLL::A, ClockOutput::Clk0, Frequency::from_hz(freq))
        .unwrap();
    clock.set_clock_drive(ClockOutput::Clk0, DriveStrength::_2mA);
    clock.set_clock_enabled(ClockOutput::Clk0, true);
    clock.flush_clock_control(ClockOutput::Clk0).unwrap();
    clock.flush_output_enabled().unwrap();

    loop {
        delay.delay_ms(1000u32);
        led.toggle();
    }
}
