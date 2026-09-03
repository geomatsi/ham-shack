//! Counts the Si5351's own 10 MHz output over a one-second gate
//!
//! The counted signal is a *clock source*, not something to input-capture: it
//! drives a TIM2 slave-mode input and clocks the counter directly.
//!
//! ```text
//!   Si5351 CLK1 (10 MHz)  ->  PB3   TIM2_CH2   counter clock, via TI2
//!   Si5351 SCL/SDA        ->  PB8/PB9         I2C1, remapped
//! ```
//!
//! PB3 is a channel pad, not ETR, so this is external clock mode **1** (SMS =
//! 111, TS = TI2FP2) rather than mode 2. Practically the two are equivalent
//! here: both resynchronize the input to CK_INT and so carry the same ceiling,
//! and mode 1 costs only a CCMR1/CCER pair on top.
//!
//! # Why two timers
//!
//! Every timer on the STM32F103 is 16 bit, TIM2 included — the 32-bit TIM2/TIM5
//! are an F2/F4/F7 part. 10 MHz over a one-second gate is 10^7 counts and needs
//! 24 bits, so the count is assembled from a cascaded pair:
//!
//! ```text
//!   TIM2  TI2  -> CNT[15:0]    master, MMS = update: TRGO on every overflow
//!   TIM3  ITR1 -> CNT[31:16]   slave, external clock mode 1 on TIM2's TRGO
//! ```
//!
//! A real 32-bit counter in hardware: no overflow bookkeeping and no wrap race,
//! because TIM3 is incremented by the very edge that wraps TIM2.
//!
//! Everything below is RM0008 Rev 20:
//!
//! - Table 45, TIM2 remapping: at TIM2_REMAP = 01, CH1_ETR is PA15 and **CH2 is
//!   PB3**. One remap value serves both, so the choice between them is a
//!   software one — see below.
//! - Table 86, internal trigger connection: for slave **TIM3, ITR1 is TIM2**
//!   (TS = 001). Worth checking against the part in hand rather than assuming —
//!   the numbering is not symmetric, and slave TIM2's own ITR1 is TIM8, which
//!   does not exist on a C8.
//! - §15.3.3, external clock mode 1: the configuration order below is the
//!   manual's own procedure, CC2S then IC2F then CC2P then SMS then TS then
//!   CEN. Its note — "the capture prescaler is not used for triggering" — is
//!   why IC2PSC is left alone, and CC2E is absent from the procedure because
//!   the trigger path does not need the capture channel enabled.
//! - Table 82's preamble: "The clock of the slave timer must be enabled prior
//!   to receiving events from the master timer" — hence TIM3 before TIM2.
//!
//! PB3 is JTDO, so `disable_jtag` has to hand it over first; SWD survives that
//! (swj_cfg goes to 0b010, not 0b100), so the probe stays attached.
//!
//! # Two things to keep an eye on
//!
//! TI2 is resynchronized to CK_INT (§15.3.3, Figure 122), so the usual
//! f_TIMxCLK/2 ceiling applies. TIM2 runs at 32 MHz here — pclk1 is 16 MHz and
//! the APB1 prescaler is not 1, so the timer clock is doubled — which puts the
//! ceiling at 16 MHz. 10 MHz clears it by 1.6x. Raising sysclk to 72 MHz would
//! make that 3.6x if the margin ever looks thin on a scope.
//!
//! The gate is `delay_ms(1000)` off SysTick, so the reading is the Si5351
//! against the Blue Pill's own 8 MHz crystal, plus whatever the software gate
//! boundaries cost. Both are ±20-30 ppm parts, so expect the count to sit some
//! hundreds off 10^7 and to wander. That is the two crystals, not a fault.

#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_rt::entry;
#[cfg(not(feature = "rtt-log"))]
use panic_halt as _;
#[cfg(feature = "rtt-log")]
use panic_rtt_target as _;
#[cfg(feature = "rtt-log")]
use rtt_target::rtt_init_print;
use si5351::{ClockOutput, DriveStrength, Frequency, PLL, Si5351, Si5351Device};
use stm32f1xx_hal::{
    i2c::{DutyCycle, Mode},
    pac,
    prelude::*,
    rcc,
    timer::Timer,
};
use wspr_beacon::wspr_log;

const PLL_FREQ: u32 = 700_000_000;
const CLK1_FREQ: u32 = 10_000_000;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cm::Peripherals::take().unwrap();
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

    // Park the PLL, then hang CLK1 off it. `set_clock_frequency_fixed_pll`
    // moves the output MultiSynth alone, so this is the same call a tone shift
    // would make later and it leaves the PLL undisturbed.
    let mut clock = Si5351Device::new_adafruit_module(i2c);
    clock.init_adafruit_module().unwrap();
    clock.select_clock_pll(ClockOutput::Clk1, PLL::A);
    clock
        .set_pll_frequency(PLL::A, Frequency::from_hz(PLL_FREQ))
        .unwrap();
    clock.reset_pll(PLL::A).unwrap();
    clock
        .set_clock_frequency_fixed_pll(ClockOutput::Clk1, Frequency::from_hz(CLK1_FREQ))
        .unwrap();
    clock.set_clock_drive(ClockOutput::Clk1, DriveStrength::_8mA);
    clock.set_clock_enabled(ClockOutput::Clk1, true);
    clock.flush_clock_control(ClockOutput::Clk1).unwrap();
    clock.flush_output_enabled().unwrap();

    // Reclaim JTDO from the debug port. PA15 (JTDI) and PB4 (JTRST) come along
    // unwanted — only PB3 is needed here. PA15 stays available though: it is
    // TIM2_CH1_ETR at this same remap, so a PPS capture could be added on it
    // later without moving anything.
    let (_pa15, pb3, _pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    // TIM2_REMAP = 01 puts CH2 on PB3.
    //
    // This has to go through the HAL's `modify_mapr` rather than the PAC.
    // MAPR's swj_cfg is write-only and reads back undefined, so a plain
    // read-modify-write can put JTAG back and take PA15 with it; `modify_mapr`
    // re-asserts the 0b010 that `disable_jtag` set. Same reason the I2C1 remap
    // above is safe.
    afio.mapr
        .modify_mapr(|_, w| unsafe { w.tim2_remap().bits(0b01) });

    // Floating: the Si5351 drives this pin and does not want a pull.
    let _clk = pb3.into_floating_input(&mut gpiob.crl);

    // The HAL has no external-clock-mode API, so both timers are driven through
    // the PAC. `Timer::new` is still worth going through — it ungates and
    // resets the peripheral — and `release` hands the register block back.
    let lo = Timer::new(dp.TIM2, &mut rcc).release();
    let hi = Timer::new(dp.TIM3, &mut rcc).release();

    // Configure both halves and start counting.
    let setup = || {
        // High half first: the slave's clock must be running before the master
        // starts sending it events.
        //
        // Full range on both. ARPE is 0 out of reset, so ARR is unbuffered and
        // this lands immediately — no update event needed to load it. That
        // matters: with MMS = update, an EGR.UG would emit a TRGO pulse and
        // miscount the high half by one. Nothing here ever writes UG. PSC is
        // left at its reset 0, so its shadow needs no reload either.
        hi.arr().write(|w| w.arr().set(u16::MAX));
        hi.smcr().write(|w| w.ts().itr1().sms().ext_clock_mode());
        hi.cr1().write(|w| w.cen().set_bit());

        // Low half, following §15.3.3's procedure for external clock mode 1 on
        // TI2. No input filter: CLK1 is a clean square wave and every edge is
        // wanted. IC2PSC is deliberately untouched — the capture prescaler
        // plays no part in triggering — and so is CC2E, since the trigger path
        // taps TI2FP2 ahead of the capture channel itself.
        lo.arr().write(|w| w.arr().set(u16::MAX));
        lo.ccmr1_input()
            .write(|w| w.cc2s().ti2().ic2f().no_filter());
        lo.ccer().write(|w| w.cc2p().clear_bit());
        lo.smcr().write(|w| w.sms().ext_clock_mode().ts().ti2fp2());
        lo.cr2().write(|w| w.mms().update());
        // URS = 1 so only a genuine overflow can ever reach TRGO.
        lo.cr1().write(|w| w.urs().set_bit().cen().set_bit());
    };

    // Stopping the master is what makes the read safe: with it halted neither
    // half can move, so the two reads cannot straddle a carry and need no
    // read-high/read-low/re-read dance.
    let stop = || {
        lo.cr1().modify(|_, w| w.cen().clear_bit());
        hi.cr1().modify(|_, w| w.cen().clear_bit());
    };

    let read = || {
        let low = lo.cnt().read().cnt().bits() as u32;
        let high = hi.cnt().read().cnt().bits() as u32;
        (high << 16) | low
    };

    let clear = || {
        lo.cnt().write(|w| w.cnt().set(0));
        hi.cnt().write(|w| w.cnt().set(0));
    };

    wspr_log!("gate: counting CLK1, nominal {} Hz", CLK1_FREQ);

    loop {
        setup();
        delay.delay_ms(1000u32);
        stop();
        let ticks = read();
        clear();

        wspr_log!(
            "gate: {} ticks ({:+} vs nominal)",
            ticks,
            ticks as i64 - CLK1_FREQ as i64
        );
    }
}
