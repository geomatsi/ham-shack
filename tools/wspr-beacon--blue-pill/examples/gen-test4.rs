#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

#[cfg(feature = "rtt-log")]
use panic_rtt_target as _;

#[cfg(feature = "rtt-log")]
use rtt_target::rtt_init_print;

#[cfg(not(feature = "rtt-log"))]
use panic_halt as _;

use cortex_m_rt::entry;
use si5351::{ClockOutput, DriveStrength, Frequency, PLL, Si5351, Si5351Device};
use stm32f1xx_hal::{
    gpio::*,
    i2c::{DutyCycle, Mode},
    pac,
    prelude::*,
    rcc,
};
use wspr_beacon::wspr_log;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.freeze(
        rcc::Config::hse(8.MHz()).sysclk(32.MHz()).pclk1(16.MHz()),
        &mut flash.acr,
    );

    #[cfg(feature = "rtt-log")]
    rtt_init_print!();

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

    let mut xmit = Si5351Device::new_adafruit_module(i2c);
    xmit.init_adafruit_module().unwrap();
    xmit.select_clock_pll(ClockOutput::Clk0, PLL::A);
    xmit.set_clock_drive(ClockOutput::Clk0, DriveStrength::_8mA);

    // disable all clocks on startup
    xmit.set_clock_enabled(ClockOutput::Clk0, false);
    xmit.set_clock_enabled(ClockOutput::Clk1, false);
    xmit.set_clock_enabled(ClockOutput::Clk2, false);

    xmit.flush_output_enabled().unwrap();
    xmit.set_pll_frequency(PLL::A, Frequency::from_hz(62 * 14_097_100))
        .unwrap();
    xmit.reset_pll(PLL::A).unwrap();

    let symbols = wspr_encoder::encode("R1BRL", "KP50", 27).unwrap();

    wspr_log!("Start test WSPR transmission");

    // 7MHz / 40m
    let dial: Frequency = Frequency::from_hz(7_038_600);
    // 14MHz / 20m
    //let dial: Frequency = Frequency::from_hz(14_095_600);
    let mut pkt: u64 = 0;

    // Slot layout, shared by the draw here and the Tx offset below. The assert
    // holds the top slot's highest tone inside the 1400-1600 Hz sub-band.
    const SLOT_COUNT: u32 = 10;
    const SLOT_BASE_HZ: u32 = 1_400;
    const SLOT_STEP_HZ: u32 = 20;
    // 4 tones spaced 1.46484375 Hz
    const SLOT_SPAN_HZ: u32 = 5;
    const _: () = assert!(SLOT_BASE_HZ + (SLOT_COUNT - 1) * SLOT_STEP_HZ + SLOT_SPAN_HZ <= 1_600);

    loop {
        // Pseudo random slot: fixed-point reduction onto 0..SLOT_COUNT, exact and total
        let h = (pkt as u32).wrapping_mul(2_654_435_761);
        let slot: u32 = ((h as u64 * SLOT_COUNT as u64) >> 32) as u32;

        const WSPR_SYMBOL_SAMPLES: u64 = 8192;
        const WSPR_SAMPLE_RATE_HZ: u64 = 12000;
        let offset = Frequency::from_hz(SLOT_BASE_HZ + slot * SLOT_STEP_HZ);

        for (num, symbol) in symbols.iter().enumerate() {
            let freq = dial
                + offset
                + Frequency::from_ratio(
                    WSPR_SAMPLE_RATE_HZ * (*symbol as u64),
                    WSPR_SYMBOL_SAMPLES as u32,
                );

            wspr_log!(
                "WSPR: transmit symbol[{}] {} at freq {}",
                num,
                symbol,
                freq.as_hz()
            );

            xmit.set_clock_frequency_fixed_pll(ClockOutput::Clk0, freq)
                .unwrap();
            delay.delay_ms(682u32);
        }

        pkt = pkt.saturating_add(1);
    }
}
