//! Counts the Si5351's 10 MHz output over a GPS-PPS gate and corrects it.
//!
//! ```text
//!   GPS PPS         ->  PB1        EXTI1      gate boundaries
//!   Si5351 CLK1     ->  PB3        TIM2_CH2   counter clock, via TI2
//!   Si5351 SCL/SDA  ->  PB8/PB9    I2C1, remapped
//! ```
//!
//! Counting is `gen-test2` unchanged: every F103 timer is 16 bit, so TIM2
//! counts CLK1 and TIM3 counts TIM2's overflows on ITR1, giving a 32-bit
//! counter in hardware. RM0008 references are in that example.
//!
//! New here is the gate. `setup` does steps 1-5 of RM0008 §15.3.3 once and CEN
//! alone is step 6, so `enable`/`disable` are one CR1 store each, reached by the
//! same path from the same spin loop. Symmetry is the point: whatever the PPS
//! handler and the wait cost, they cost the same at both ends and cancel out of
//! the difference. One microsecond of asymmetry would be 1 ppm at this gate.
//!
//! A gate runs on alternate seconds — arm, wait, open; arm, wait, close; then
//! read and correct with the counters stopped. An edge arriving during that
//! processing is ignored, so a gate can never open short.
//!
//! Resolution is one tick, 100 ppb, hence the 300 ppb threshold: anything
//! tighter chases quantization.
//!
//! The correction comes from `si5351::calibrate` and is applied here to the
//! counted output itself. So CLK1 stops dividing 700 MHz exactly
//! and the driver's rounding joins the loop, well under a tick; and the loop is
//! closed, so the linearisation's residual is absorbed by the next gate. The
//! beacon proper counts an integer-divider CLK1 and corrects CLK0 — the same
//! operation, since the fractional error is shared.
//!
//! # Limits
//!
//! One calibration is a snapshot, applied once and then held while the crystal
//! moves. Landing in the WSPR sub-band needs ±7.1 ppm, so 100 ppb of gate
//! resolution is already ~70x finer than required: what bounds the result is
//! drift *after* the correction, which no gate length reaches. A longer gate
//! would only return a mean over a longer window, staler when applied.
//!
//! For the beacon proper, roughly by value per effort:
//!
//! - keep the module powered and enclosed — warm-up and draughts dominate
//! - recalibrate immediately before each transmission: removes aging and the
//!   day/night excursion, nothing within the frame
//! - correct at symbol boundaries from a CLK1 gate on the same parked PLL, the
//!   only option that reaches drift during a transmission
//! - a TCXO on XA, which removes the problem rather than tracking it

#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

#[cfg(feature = "rtt-log")]
use panic_rtt_target as _;

#[cfg(feature = "rtt-log")]
use rtt_target::rtt_init_print;

#[cfg(not(feature = "rtt-log"))]
use panic_halt as _;

use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m_rt::entry;
use pac::interrupt;
use si5351::{ClockOutput, DriveStrength, Frequency, PLL, Si5351, Si5351Device, calibrate};
use stm32f1xx_hal::{
    gpio::*,
    i2c::{DutyCycle, Mode},
    pac,
    prelude::*,
    rcc,
    timer::Timer,
};
use wspr_beacon::wspr_log;

static PPS_EVT: AtomicBool = AtomicBool::new(false);

const CLK1_FREQ: u32 = 10_000_000;
const NOMINAL: Frequency = Frequency::from_hz(CLK1_FREQ);
const PLL_FREQ: u32 = 700_000_000;

#[entry]
fn main() -> ! {
    let mut dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.freeze(
        rcc::Config::hse(8.MHz()).sysclk(32.MHz()).pclk1(16.MHz()),
        &mut flash.acr,
    );

    #[cfg(feature = "rtt-log")]
    rtt_init_print!();

    let mut afio = dp.AFIO.constrain(&mut rcc);
    let gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);

    // GPS PPS

    // The handler clears EXTI PR directly, so the pin is not kept.
    let mut pps = gpiob.pb1.into_floating_input(&mut gpiob.crl);
    pps.make_interrupt_source(&mut afio);
    pps.trigger_on_edge(&mut dp.EXTI, Edge::Rising);
    pps.enable_interrupt(&mut dp.EXTI);

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::EXTI1);
    }

    // GEN SI5351

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
    let mut freq = Frequency::from_hz(CLK1_FREQ);

    clock.init_adafruit_module().unwrap();

    // Driver state only; flushed by set_clock_frequency_fixed_pll below.
    clock.select_clock_pll(ClockOutput::Clk1, PLL::A);
    clock.set_clock_drive(ClockOutput::Clk1, DriveStrength::_8mA);

    clock
        .set_pll_frequency(PLL::A, Frequency::from_hz(PLL_FREQ))
        .unwrap();
    clock.reset_pll(PLL::A).unwrap();
    // Enables the output and flushes control and OE itself.
    clock
        .set_clock_frequency_fixed_pll(ClockOutput::Clk1, freq)
        .unwrap();

    let (_pa15, pb3, _pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    afio.mapr
        .modify_mapr(|_, w| unsafe { w.tim2_remap().bits(0b01) });

    let _clk = pb3.into_floating_input(&mut gpiob.crl);

    let lo = Timer::new(dp.TIM2, &mut rcc).release();
    let hi = Timer::new(dp.TIM3, &mut rcc).release();

    let setup = || {
        hi.arr().write(|w| w.arr().set(u16::MAX));
        hi.smcr().write(|w| w.ts().itr1().sms().ext_clock_mode());
        // The high half runs continuously. It costs nothing between gates: a
        // stopped TIM2 emits no update and so no TRGO, so TIM3 sees no clock.
        hi.cr1().write(|w| w.cen().set_bit());

        lo.arr().write(|w| w.arr().set(u16::MAX));
        lo.ccmr1_input()
            .write(|w| w.cc2s().ti2().ic2f().no_filter());
        lo.ccer().write(|w| w.cc2p().clear_bit());
        lo.smcr().write(|w| w.sms().ext_clock_mode().ts().ti2fp2());
        lo.cr2().write(|w| w.mms().update());
        // URS = 1 so only a genuine overflow can ever reach TRGO.
        lo.cr1().write(|w| w.urs().set_bit());
    };

    let enable = || lo.cr1().write(|w| w.urs().set_bit().cen().set_bit());
    let disable = || lo.cr1().write(|w| w.urs().set_bit().cen().clear_bit());

    let read = || {
        let low = lo.cnt().read().cnt().bits() as u32;
        let high = hi.cnt().read().cnt().bits() as u32;
        (high << 16) | low
    };

    let clear = || {
        lo.cnt().write(|w| w.cnt().set(0));
        hi.cnt().write(|w| w.cnt().set(0));
    };

    // get ready: setup and clear
    setup();
    clear();

    wspr_log!("gate: counting CLK1, nominal {} Hz", CLK1_FREQ);
    wspr_log!("waiting for the pps fix...");

    // wait for the 1st pps
    PPS_EVT.store(true, Ordering::Relaxed);
    while PPS_EVT.load(Ordering::Acquire) {
        cortex_m::asm::nop();
    }

    wspr_log!("start measurements");

    let mut c: u8 = 0;

    loop {
        // open the gate on a second boundary
        PPS_EVT.store(true, Ordering::Relaxed);
        while PPS_EVT.load(Ordering::Acquire) {
            cortex_m::asm::nop();
        }

        enable();

        // close it on the next one
        PPS_EVT.store(true, Ordering::Relaxed);
        while PPS_EVT.load(Ordering::Acquire) {
            cortex_m::asm::nop();
        }

        disable();
        let ticks = read();
        clear();

        let ppb = calibrate::error_ppb(ticks, NOMINAL, 1);
        wspr_log!(
            "gate: {} ticks ({:+} vs nominal) ppb {}",
            ticks,
            ticks as i64 - CLK1_FREQ as i64,
            ppb
        );

        // No crystal is out by 200 ppm, so this is a lost or doubled PPS rather
        // than a measurement.
        if !calibrate::plausible(ppb) {
            wspr_log!("|pbb| {} is too large to be true, gate rejected", ppb.abs());
            continue;
        }

        // One tick of a 1 s gate on 10 MHz is 100 ppb, so anything tighter than
        // a few ticks would just chase quantization noise.
        // Let si5351 stabilize for six gates before trying next correction —
        // 12 s, since a gate only runs on alternate seconds.
        // Note: a bit more advanced math such as averaging is deliberately
        // avoided in this simple example.
        if c > 5 && ppb.abs() > 300 {
            freq = calibrate::correct(freq, ppb);
            wspr_log!("apply correction to freq {} uHz", freq.as_microhz());
            clock
                .set_clock_frequency_fixed_pll(ClockOutput::Clk1, freq)
                .unwrap();
            c = 0;
        }

        c = c.saturating_add(1);
    }
}

#[interrupt]
fn EXTI1() {
    PPS_EVT.store(false, Ordering::Release);
    // PB1 is the only source on this line, so no need to check PR first.
    unsafe {
        (*pac::EXTI::ptr())
            .pr()
            .write(|w| w.pr1().clear_bit_by_one())
    };
}
