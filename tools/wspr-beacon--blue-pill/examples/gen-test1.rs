#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_rt::entry;
use panic_semihosting as _;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::rcc;
use wspr_beacon::support::si5351::{ClockOutput, PLL, Si5351, Si5351Device};

use stm32f1xx_hal::{
    i2c::{DutyCycle, Mode},
    prelude::*,
};

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

    let gpiob = dp.GPIOB.split(&mut rcc);
    let mut delay = cp.SYST.delay(&rcc.clocks);

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

    let freqs = [7_000_000u32, 8_000_000u32, 9_000_000u32, 10_000_000u32];

    loop {
        for freq in freqs {
            clock
                .set_frequency(PLL::A, ClockOutput::Clk0, freq)
                .unwrap();
            clock.set_clock_enabled(ClockOutput::Clk0, true);
            clock.flush_clock_control(ClockOutput::Clk0).unwrap();
            clock.flush_output_enabled().unwrap();

            delay.delay_ms(1000u32);
        }
    }
}
