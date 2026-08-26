#![no_main]
#![no_std]

#[cfg(feature = "rtt-log")]
use panic_rtt_target as _;

#[cfg(not(feature = "rtt-log"))]
use panic_halt as _;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [SPI1, SPI2])]
mod app {
    use core::sync::atomic::{AtomicBool, Ordering, compiler_fence};
    use rtic_monotonics::rtic_time::monotonic::TimerQueueBasedInstant;
    use stm32f1xx_hal::i2c::BlockingI2c;
    use wspr_beacon::beacon::calibration::{Calibration, Reading, Step};
    use wspr_beacon::beacon::config::CFG;
    use wspr_beacon::beacon::events::Event;
    use wspr_beacon::beacon::qth::{Coordinates, qth_square};
    use wspr_beacon::beacon::states::{ErrorState, State, WSPRError};
    use wspr_beacon::beacon::status::{DisplayInfo, Status, Time};
    use wspr_beacon::wspr_log;
    use wspr_beacon::{dwt_since, dwt_stamp, dwt_start};

    use wspr_beacon::hw::display::{self, StatusDisplay};

    #[cfg(feature = "rtt-log-debug")]
    use wspr_beacon::wspr_lognln;

    use cortex_m::singleton;
    use heapless::binary_heap::{BinaryHeap, Max};
    use nmea0183;
    use rtic_monotonics::stm32::prelude::*;
    use si5351::{ClockOutput, Frequency, Si5351, Si5351Device, calibrate};
    use stm32f1xx_hal::{
        dma::CircBuffer,
        gpio::{self, Edge, ExtiPin, Input},
        i2c::{DutyCycle, Mode},
        pac::{self, USART3},
        prelude::*,
        serial,
        timer::Timer,
        watchdog,
    };
    use wspr_encoder;

    // DMA1 is only touched by the rtt-log-debug byte dump in the GPS handler.
    #[cfg(feature = "rtt-log-debug")]
    use stm32f1xx_hal::pac::DMA1;

    #[cfg(feature = "rtt-log")]
    use rtt_target::rtt_init_print;

    static PPS_WSPR_XMIT_GATE: AtomicBool = AtomicBool::new(false);
    static CALIB_PPS_EVT_GATE: AtomicBool = AtomicBool::new(false);

    stm32_tim4_monotonic!(Mono, 10_000);

    #[shared]
    struct Shared {
        status: Status,
        queue: BinaryHeap<Event, Max, 8>,
        xmit: Si5351Device<BlockingI2c<pac::I2C1>>,
    }

    #[local]
    struct Local {
        // GPS task
        circ: Option<CircBuffer<[u8; CFG.hw.gps.ublox_len], serial::RxDma3>>,
        parser: nmea0183::Parser,

        // PPS task
        pps: gpio::gpiob::PB1<Input>,

        // Display task
        display: Option<display::Display>,

        // PPS task
        tim2: pac::TIM2,
        tim3: pac::TIM3,

        // Idle task
        wdg: watchdog::IndependentWatchdog,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local) {
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.freeze(
            stm32f1xx_hal::rcc::Config::hse(CFG.hw.mcu.crystal_mhz.MHz())
                .sysclk(CFG.hw.mcu.sysclk_mhz.MHz())
                .pclk1(CFG.hw.mcu.pclk_mhz.MHz()),
            &mut flash.acr,
        );

        #[cfg(feature = "dwt-profile")]
        {
            cx.core.DCB.enable_trace();
            cx.core.DWT.enable_cycle_counter();
        }

        #[cfg(feature = "rtt-log")]
        rtt_init_print!();

        let mut afio = cx.device.AFIO.constrain(&mut rcc);
        let gpioa = cx.device.GPIOA.split(&mut rcc);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc);
        let channels = cx.device.DMA1.split(&mut rcc);

        //// Init and start watchdog early to catch init issues as well
        let mut wdg = watchdog::IndependentWatchdog::new(cx.device.IWDG);
        wdg.stop_on_debug(&cx.device.DBGMCU, true);
        wdg.start(CFG.sw.wdg.period_ms.millis());

        //// Si5351 generator
        let scl = gpiob.pb8;
        let sda = gpiob.pb9;

        let i2c = cx.device.I2C1.remap(&mut afio.mapr).blocking_i2c(
            (scl, sda),
            Mode::Fast {
                frequency: CFG.hw.mcu.i2c_khz.kHz(),
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
        xmit.select_clock_pll(CFG.hw.rf.wspr_clk, CFG.hw.rf.pll);
        xmit.select_clock_pll(CFG.hw.rf.calib_clk, CFG.hw.rf.pll);
        xmit.set_clock_drive(CFG.hw.rf.wspr_clk, CFG.hw.rf.drive);
        xmit.set_clock_drive(CFG.hw.rf.calib_clk, CFG.hw.rf.drive);
        // disable all clocks on startup
        xmit.set_clock_enabled(ClockOutput::Clk0, false);
        xmit.set_clock_enabled(ClockOutput::Clk1, false);
        xmit.set_clock_enabled(ClockOutput::Clk2, false);
        xmit.flush_output_enabled().unwrap();
        xmit.set_pll_frequency(CFG.hw.rf.pll, CFG.hw.rf.pll_parked)
            .unwrap();
        xmit.reset_pll(CFG.hw.rf.pll).unwrap();

        // Calibration task
        let (_pa15, pb3, _pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);
        afio.mapr
            .modify_mapr(|_, w| unsafe { w.tim2_remap().bits(0b01) });
        let _clk = pb3.into_floating_input(&mut gpiob.crl);
        let tim2 = Timer::new(cx.device.TIM2, &mut rcc).release();
        let tim3 = Timer::new(cx.device.TIM3, &mut rcc).release();

        tim3.arr().write(|w| w.arr().set(u16::MAX));
        tim3.smcr().write(|w| w.ts().itr1().sms().ext_clock_mode());
        tim3.cr1().write(|w| w.cen().set_bit());

        tim2.arr().write(|w| w.arr().set(u16::MAX));
        tim2.ccmr1_input()
            .write(|w| w.cc2s().ti2().ic2f().no_filter());
        tim2.ccer().write(|w| w.cc2p().clear_bit());
        tim2.smcr()
            .write(|w| w.sms().ext_clock_mode().ts().ti2fp2());
        tim2.cr2().write(|w| w.mms().update());
        tim2.cr1().write(|w| w.urs().set_bit());

        //// Display
        #[cfg(not(feature = "display"))]
        let display: Option<display::Display> = None;

        #[cfg(feature = "display")]
        let display = {
            display::create(
                display::DisplayParts {
                    crl: gpioa.crl,
                    pa0: gpioa.pa0,
                    pa1: gpioa.pa1,
                    pa2: gpioa.pa2,
                    pa3: gpioa.pa3,
                    pa4: gpioa.pa4,
                    pa5: gpioa.pa5,
                    tim: cx.device.TIM1,
                },
                &mut rcc,
            )
        };

        //// PPS
        let mut pps = gpiob.pb1.into_floating_input(&mut gpiob.crl);
        pps.make_interrupt_source(&mut afio);
        pps.trigger_on_edge(&mut cx.device.EXTI, Edge::Rising);
        pps.enable_interrupt(&mut cx.device.EXTI);

        //// GPS
        let stx = gpiob.pb10.into_alternate_open_drain(&mut gpiob.crh);
        let srx = gpiob.pb11;
        let (_, mut rx) = cx
            .device
            .USART3
            .remap(&mut afio.mapr)
            .serial((stx, srx), CFG.hw.gps.baudrate.bps(), &mut rcc)
            .split();

        // setup serial 'idle' interrupt before converting rx into rxdma
        rx.listen_idle();

        let nmea_parser = nmea0183::Parser::new()
            .sentence_filter(nmea0183::Sentence::RMC | nmea0183::Sentence::GGA);
        let dmabuf =
            singleton!(: [[u8; CFG.hw.gps.ublox_len]; 2] = [[0; CFG.hw.gps.ublox_len]; 2]).unwrap();
        let rxdma = rx.with_dma(channels.3);
        let circ = rxdma.circ_read(dmabuf);

        //// shared globals

        let queue: BinaryHeap<Event, Max, 8> = BinaryHeap::new();
        let status = Status::new();

        //// Interrupts

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::USART3);
            pac::NVIC::unmask(pac::Interrupt::EXTI1);
        }

        Mono::start(rcc.clocks.pclk1_tim().to_Hz());

        display_task::spawn().unwrap();
        wdg_task::spawn().unwrap();

        (
            Shared {
                status,
                queue,
                xmit,
            },
            Local {
                // GPS task
                circ: Some(circ),
                parser: nmea_parser,

                // PPS task
                pps,

                // Display task
                display,

                // PPS task
                tim2,
                tim3,

                // IDLE task
                wdg,
            },
        )
    }

    #[idle(local = [wdg], shared = [status, queue, xmit])]
    fn idle(mut cx: idle::Context) -> ! {
        let mut calib: Option<Calibration> = None;
        // Calibration timeout is monitored by BEATs, so derive limit from a BEAT duration `wdg.feed_ms`.
        const NOPPS_CALIB_MS: u64 = 20_000;
        const NOPPS_CALIB_LIMIT: u16 = (NOPPS_CALIB_MS / CFG.sw.wdg.feed_ms) as u16;

        let mut nopps_calib_tick: u16 = 0;
        const NOGPS_LOG_PERIOD: u16 = 20;
        let mut nogps_log_tick: u16 = 0;

        loop {
            // Drain every queued event before sleeping. Popping a single event
            // per WFI can strand a second queued event for up to a second, since
            // it would need an unrelated interrupt to wake the core again.
            loop {
                let mut event: Option<Event> = None;
                cx.shared.queue.lock(|queue| {
                    compiler_fence(Ordering::SeqCst);
                    event = queue.pop();
                });

                let Some(event) = event else {
                    break;
                };

                // Event handling: benchmarking
                match event {
                    Event::GPS(_, _) => dwt_stamp!("Event GPS"),
                    Event::NOGPS => dwt_stamp!("Event NOGPS"),
                    _ => {}
                }

                // Event handling: fill displayed information
                // - QTH is not maintained here: it is filled in the FSM where WSPR is encoded
                if let Event::GPS(_, (hours, minutes, _)) = event {
                    cx.shared.status.lock(|status| {
                        status.time = Some(Time { hours, minutes });
                    });
                }

                // Event handling: wdg feed
                if matches!(event, Event::BEAT) {
                    cx.local.wdg.feed();
                }

                // Event handling: main FSM
                cx.shared.status.lock(|status| match status.state {
                    State::GpsWait => match event {
                        Event::GPS((lat, lon), _) => {
                            wspr_log!("SCHED: GPS coords ({}, {})", lat as u8, lon as u8);

                            let coords = Coordinates {
                                latitude: lat,
                                longitude: lon,
                            };
                            let mut qth_buf: [u8; 4] = [0, 0, 0, 0];
                            dwt_start!(qth_start);

                            let encoded = match qth_square(coords, &mut qth_buf) {
                                Ok(qth) => {
                                    dwt_since!("SCHED: QTH calculation", qth_start);
                                    wspr_log!("SCHED: calculated QTH {}", qth);

                                    dwt_start!(enc_start);

                                    match wspr_encoder::encode(CFG.ham.callsign, qth, CFG.ham.pwr) {
                                        Ok(symbols) => {
                                            dwt_since!("SCHED: WSPR encoding", enc_start);

                                            Some(symbols)
                                        }
                                        Err(e) => {
                                            wspr_log!(
                                                "SCHED: fatal WSPR encoding failure: {:?}",
                                                e
                                            );

                                            None
                                        }
                                    }
                                }
                                Err(e) => {
                                    wspr_log!("SCHED: fatal QTH calculation failure: {:?}", e);

                                    None
                                }
                            };

                            // Published together: `qth` borrows `qth_buf` until the match ends.
                            if let Some(symbols) = encoded {
                                status.msg = Some(symbols);
                                status.qth = Some(qth_buf);
                                status.state = State::TxWait;
                            }
                        }
                        Event::NOGPS => {
                            if nogps_log_tick == 0 {
                                wspr_log!("SCHED: no GPS fix in GpsWait");
                            }
                            nogps_log_tick = (nogps_log_tick + 1) % NOGPS_LOG_PERIOD;
                        }
                        _ => {}
                    },
                    State::TxWait => match event {
                        Event::GPS(_, (_, min, sec)) => {
                            let sec = sec as u8;

                            if sec.is_multiple_of(10) {
                                wspr_log!("SCHED: TxWait: GPS: Time ({}:{})", min, sec);
                            }

                            // Calibrate at :10 of every calib_period_min-th minute.
                            if sec == 10u8 && min.is_multiple_of(CFG.ham.calib_period_min) {
                                wspr_log!("SCHED: started calibration");
                                CALIB_PPS_EVT_GATE.store(true, Ordering::Release);
                                calib = Some(Calibration::new(CFG.hw.rf.nominal));
                                status.state = State::TxCalib;
                                nopps_calib_tick = 0;
                            }

                            // Spawn at :00 so the next PPS edge is the :01 a
                            // frame starts on - the task needs to know nothing
                            // more. tx_period_min is even, which is what keeps
                            // frames on the even minutes receivers listen on.
                            if sec == 0u8
                                && min.is_multiple_of(CFG.ham.tx_period_min)
                                && status.ppb.is_some()
                            {
                                match wspr::spawn() {
                                    Ok(_) => {
                                        wspr_log!("SCHED: spawned WSPR");
                                        status.state = State::TxActive;
                                    }
                                    Err(_) => {
                                        wspr_log!("SCHED: failed to spawn WSPR");
                                        // TODO: ignore and wait for the next slot or reset state ?
                                    }
                                }
                            }
                        }
                        Event::NOGPS => {
                            wspr_log!("SCHED: GPS lost in TxWait");
                            status.reset();
                        }
                        _ => {}
                    },
                    State::TxCalib => match event {
                        Event::BEAT => {
                            nopps_calib_tick = nopps_calib_tick.saturating_add(1);
                            // PPS gates land every 2 s: silence means PPS or the path to it
                            // died, leaving RF up from the first Step::SetDial.
                            if nopps_calib_tick >= NOPPS_CALIB_LIMIT {
                                wspr_log!("SCHED: no PPS-gated measurement during calibration");
                                // emergency exit from TxCalib state
                                status.state = State::Error(ErrorState::CalibFailure);
                            }
                        }
                        Event::CALIB(ticks) => {
                            nopps_calib_tick = 0;
                            let result = 'calib: {
                                if let Some(c) = calib.as_mut() {
                                    let step = c.gate(ticks);

                                    match c.last_reading() {
                                        Reading::Ppb(ppb) => {
                                            wspr_log!(
                                                "SCHED: calibraton: {} ticks ({:+} vs nominal) ppb {}",
                                                ticks,
                                                ticks as i64 - CFG.hw.rf.nominal.as_hz() as i64,
                                                ppb
                                            );
                                        }
                                        Reading::Rejected(ppb) => {
                                            wspr_log!(
                                                "SCHED: calibration: |pbb| {} is too large to be true, gate rejected",
                                                ppb.abs()
                                            );
                                        }
                                        Reading::Skipped => {}
                                    }

                                    match step {
                                        Step::Continue => {}
                                        Step::SetDial(dial) => {
                                            wspr_log!(
                                                "SCHED: calibration: set dial freq {} uHz",
                                                dial.as_microhz()
                                            );
                                            if let Err(e) = cx.shared.xmit.lock(|xmit| {
                                                xmit.set_clock_frequency_fixed_pll(CFG.hw.rf.calib_clk, dial)
                                            }) {
                                                wspr_log!("SCHED: failed to change frequency for calibration: {}", e);
                                                break 'calib Err(e.into());
                                            }
                                        }
                                        Step::Stop(result) => {
                                            if let Err(e) = cx.shared.xmit.lock(|xmit| {
                                                xmit.set_clock_enabled(CFG.hw.rf.calib_clk, false);
                                                xmit.flush_output_enabled()
                                            }) {
                                                wspr_log!("SCHED: failed to stop RF after calibration: {}", e);
                                                break 'calib Err(e.into());
                                            }

                                            match result {
                                                Some(ppb) => {
                                                    wspr_log!("SCHED: calibration completed with {} ppb", ppb)
                                                }
                                                None => {
                                                    wspr_log!("SCHED: calibration failed: no usable gate")
                                                }
                                            }

                                            CALIB_PPS_EVT_GATE.store(false, Ordering::Release);
                                            status.state = State::TxWait;
                                            status.ppb = result;
                                            calib = None;
                                        }
                                    }
                                } else {
                                    // No calibration in flight: nothing sane to do with the tick.
                                    // Stop the clock and start over.
                                    wspr_log!("SCHED: unexpected calibration state... Stop and reset...");
                                    break 'calib Err(WSPRError::CalibUnexpectedState);
                                }

                                Ok(())
                            };

                            if result.is_err() {
                                wspr_log!("SCHED: calibration failed");
                                // RF teardown is State::Error's job, one BEAT away at most.
                                status.state = State::Error(ErrorState::CalibFailure);
                            }
                        }
                        _ => {}
                    }
                    State::TxActive => {
                        if let Event::TXDONE = event {
                            // Start over from a fresh fix before the next transmission
                            wspr_log!("SCHED: Tx completed");
                            status.reset();
                        }
                    }
                    State::Error(code) => {
                        wspr_log!("SCHED: error code {}", code.as_str());
                        CALIB_PPS_EVT_GATE.store(false, Ordering::Release);
                        // NB: only an attempt - same I2C path that may have just failed.
                        // In reliable h/w design we should be able to
                        // power off the si5351 generator
                        cx.shared.xmit.lock(|xmit| {
                            xmit.set_clock_enabled(CFG.hw.rf.calib_clk, false);
                            xmit.set_clock_enabled(CFG.hw.rf.wspr_clk, false);
                            xmit.flush_output_enabled().ok();
                        });
                        calib = None;
                        status.reset();
                        // TODO: add last error state to Status and display it for diagnostic purposes
                    }
                });
            }

            // Queue drained. Re-check emptiness before sleeping so an event queued
            // during the last drain iteration is picked up immediately instead of
            // waiting for the next wake.
            //
            // No critical section is needed to fully close the residual
            // check-then-WFI race: the TIM4 monotonic keeps two always-on periodic
            // interrupts running (full- and half-period, see the rtic-monotonics
            // stm32 backend) that wake the core regardless of any pending `await`.
            // At the current 100 kHz tick rate the 16-bit timer wraps every
            // 655.36 ms, so those land every ~328 ms. An event queued in the small
            // window between this check and the WFI is therefore drained within
            // that, still inside the 1 Hz GPS/PPS cadence, so no event is stranded
            // in practice.
            let empty = cx.shared.queue.lock(|queue| {
                compiler_fence(Ordering::SeqCst);
                queue.is_empty()
            });
            if empty {
                // Keep the core awake so host-side RTT attach does not time out.
                #[cfg(feature = "rtt-log")]
                cortex_m::asm::nop();

                #[cfg(not(feature = "rtt-log"))]
                cortex_m::asm::wfi();
            }
        }
    }

    #[task(priority = 1, shared = [queue])]
    async fn wdg_task(mut cx: wdg_task::Context) {
        wspr_log!("WDG started");

        loop {
            cx.shared.queue.lock(|queue| {
                if queue.push(Event::BEAT).is_err() {
                    wspr_log!("WDG: failed to send BEAT event");
                }
            });

            Mono::delay(CFG.sw.wdg.feed_ms.millis()).await;
        }
    }

    #[task(priority = 10, shared = [queue, status, xmit])]
    async fn wspr(mut cx: wspr::Context) {
        // Pseudo random slot selection
        let h = (Mono::now().ticks() as u32).wrapping_mul(2_654_435_761);
        let slot: u32 = ((h as u64 * 10) >> 32) as u32;

        wspr_log!("WSPR: started in slot {}", slot);

        let result =
            'xmit: {
                let mut ts = Mono::now();

                // spawned at :00, so the next PPS edge is the :01 a frame starts on
                PPS_WSPR_XMIT_GATE.store(true, Ordering::Relaxed);
                while PPS_WSPR_XMIT_GATE.load(Ordering::Acquire) {
                    Mono::delay(1u64.millis()).await;
                    let elapsed_usecs = (Mono::now() - ts).to_micros();
                    if elapsed_usecs > 1_200_000 {
                        wspr_log!("WSPR: failed to start Tx due to lost PPS");
                        break 'xmit Err(WSPRError::TxPPSError);
                    }
                }

                // Only the message is copied out, not the whole `Status`: the symbols
                // are needed for the next two minutes, the rest of the struct is not.
                let msg = cx.shared.status.lock(|status| status.msg);
                let ppb = cx.shared.status.lock(|status| status.ppb);

                if let Some((symbols, ppb)) = msg.zip(ppb) {
                    // WSPR modulation is defined by 8192-sample symbols at a 12 kHz rate,
                    // giving a symbol period of exactly 8192/12000 s (682.667 ms) and a
                    // tone spacing of exactly 12000/8192 Hz (1.46484375 Hz).
                    const WSPR_SYMBOL_SAMPLES: u64 = 8192;
                    const WSPR_SAMPLE_RATE_HZ: u64 = 12000;
                    let dial: Frequency = CFG.ham.bands[CFG.ham.band].dial;
                    // WSPR audio offset is 1.4KHz + 20 * slot  above the dial frequency with slot = 0..9
                    let offset = Frequency::from_hz(1_400 + slot * 20);

                    // derive each deadline from ts
                    ts = Mono::now();

                    for (num, symbol) in symbols.iter().enumerate() {
                        // Absolute deadline for the end of this symbol, computed from
                        // ts with integer math. Deriving each deadline straight from
                        // ts — rather than summing a rounded per-symbol duration —
                        // keeps the error per boundary instead of letting it accumulate:
                        // a rounded 683 ms period would drift ~54 ms by symbol 161.
                        let elapsed_us = WSPR_SYMBOL_SAMPLES * (num as u64 + 1) * 1_000_000u64
                            / WSPR_SAMPLE_RATE_HZ;
                        let deadline = ts + elapsed_us.micros();

                        let mut freq = dial
                            + offset
                            + Frequency::from_ratio(
                                WSPR_SAMPLE_RATE_HZ * (*symbol as u64),
                                WSPR_SYMBOL_SAMPLES as u32,
                            );
                        freq = calibrate::correct(freq, ppb);

                        #[cfg(feature = "rtt-log-debug")]
                        wspr_log!(
                            "WSPR: transmit symbol[{}] {} at freq {} with duration {:?}",
                            num,
                            symbol,
                            freq.as_hz(),
                            deadline - Mono::now()
                        );

                        if let Err(e) = cx.shared.xmit.lock(|xmit| {
                            xmit.set_clock_frequency_fixed_pll(CFG.hw.rf.wspr_clk, freq)
                        }) {
                            wspr_log!("WSPR: failed to set new frequency: {}", e);
                            break 'xmit Err(e.into());
                        }

                        Mono::delay_until(deadline).await;
                    }
                } else {
                    wspr_log!("WSPR: empty wspr message or empty ppb not expected");
                }

                if let Err(e) = cx.shared.xmit.lock(|xmit| {
                    xmit.set_clock_enabled(CFG.hw.rf.wspr_clk, false);
                    xmit.flush_output_enabled()
                }) {
                    wspr_log!("WSPR: failed to disable RF: {}", e);
                    break 'xmit Err(e.into());
                }

                if cx
                    .shared
                    .queue
                    .lock(|queue| queue.push(Event::TXDONE))
                    .is_err()
                {
                    wspr_log!("WSPR: failed to send TXDONE");
                    break 'xmit Err(WSPRError::TxQueueError);
                }

                Ok(())
            };

        if result.is_err() {
            wspr_log!("WSPR: transmission failed");

            // NB: here we just attempt once more to disable RF
            // in reliable h/w design we should be able to
            // power off the si5351 generator
            cx.shared.xmit.lock(|xmit| {
                xmit.set_clock_enabled(CFG.hw.rf.wspr_clk, false);
                xmit.flush_output_enabled().ok();
            });

            // emergency exit from TxActive state
            cx.shared.status.lock(|status| {
                status.state = State::Error(ErrorState::WSPRTxFailure);
            });
        }
    }

    #[task(binds = EXTI1, priority = 15, local = [pps, tim2, tim3], shared = [status, queue])]
    fn pps(mut cx: pps::Context) {
        let lo = cx.local.tim2;
        let hi = cx.local.tim3;

        let enable = || lo.cr1().write(|w| w.urs().set_bit().cen().set_bit());
        let disable = || lo.cr1().write(|w| w.urs().set_bit().cen().clear_bit());
        let is_running = || lo.cr1().read().cen().is_enabled();

        let read = || {
            let low = lo.cnt().read().cnt().bits() as u32;
            let high = hi.cnt().read().cnt().bits() as u32;
            (high << 16) | low
        };

        let clear = || {
            lo.cnt().write(|w| w.cnt().set(0));
            hi.cnt().write(|w| w.cnt().set(0));
        };

        if cx.local.pps.check_interrupt() {
            PPS_WSPR_XMIT_GATE.store(false, Ordering::Release);
            cx.local.pps.clear_interrupt_pending_bit();

            // mini FSM for reading calibration measurements every other second
            if is_running() {
                disable();
                let ticks = read();
                clear();

                // use atomic gate instead of checking state to avoid state lock
                if CALIB_PPS_EVT_GATE.load(Ordering::Acquire) {
                    cx.shared.queue.lock(|queue| {
                        // queue lock is less invasive
                        if queue.push(Event::CALIB(ticks)).is_err() {
                            wspr_log!("PPS: failed to send Calibration event");
                            // emergency exit from TxCalib state
                            cx.shared.status.lock(|status| {
                                status.state = State::Error(ErrorState::PPSQueueFailure);
                            });
                        }
                    });
                }
            } else {
                enable();
            }

            dwt_stamp!("IRQ PPS");
        }
    }

    #[task(binds = USART3, priority = 5, local = [circ, parser], shared = [status, queue])]
    fn gps(mut cx: gps::Context) {
        let mut process_nmea = true;

        // Note: rx is 'moved' on rx.with_dma, so we can not use rx anymore.
        // IIUC there is no legitimate way to use rx.is_idle together with rxdma in current stm32f1xx HAL code.
        // For now just use direct unsafe access to USART3 and DMA1 regs to check interrupt status and transferred bytes.
        let usart3 = unsafe { &*USART3::ptr() };
        if usart3.sr().read().idle().bit_is_set() {
            // clear flag — read SR then DR sequence
            let _ = usart3.sr().read();
            let _ = usart3.dr().read();

            dwt_stamp!("IRQ GPS");

            if let Some(circ) = cx.local.circ.take() {
                let (buf, rxdma) = circ.stop();

                #[cfg(feature = "rtt-log-debug")]
                {
                    let recv = (CFG.hw.gps.ublox_len * 2)
                        - unsafe { (*DMA1::ptr()).ch3().ndtr().read().ndt().bits() as usize };
                    buf[0][..recv.min(CFG.hw.gps.ublox_len)]
                        .iter()
                        .for_each(|&b| wspr_lognln!("{}", b as char));
                }

                cx.shared.status.lock(|status| {
                    if matches!(status.state, State::TxActive | State::TxCalib) {
                        process_nmea = false;
                    }
                });

                if process_nmea {
                    dwt_start!(start);

                    let mut fix = false;
                    let mut lat: f64 = 0f64;
                    let mut lon: f64 = 0f64;
                    let mut h = 0;
                    let mut m = 0;
                    let mut s = 0f32;

                    for result in cx.local.parser.parse_from_bytes(&buf[0][..]) {
                        match result {
                            Ok(nmea0183::ParseResult::RMC(Some(rmc))) => {
                                h = rmc.datetime.time.hours;
                                m = rmc.datetime.time.minutes;
                                s = rmc.datetime.time.seconds;
                                lat = rmc.latitude.as_f64();
                                lon = rmc.longitude.as_f64();
                                fix = true;
                            }
                            Ok(nmea0183::ParseResult::RMC(None)) => {
                                fix = false;
                            }
                            Ok(_) => {
                                // skip other messages for now
                            }
                            Err(e) => {
                                wspr_log!("Error parsing NMEA: {}", e);
                            }
                        }
                    }

                    cx.shared.queue.lock(|queue| {
                        if fix {
                            if queue.push(Event::GPS((lat, lon), (h, m, s))).is_err() {
                                wspr_log!("IRQ GPS: failed to send GPS event");
                            }
                        } else {
                            if queue.push(Event::NOGPS).is_err() {
                                wspr_log!("IRQ GPS: failed to send NOGPS event");
                            }
                        }
                    });

                    dwt_since!("IRQ GPS: NMEA processing", start);
                }

                buf[0].fill(0);
                *cx.local.circ = Some(rxdma.circ_read(buf));
            }
        }
    }

    #[task(priority = 1, shared = [status], local = [display])]
    async fn display_task(mut cx: display_task::Context) {
        let Some(display) = cx.local.display.as_mut() else {
            return;
        };

        let mut shown: Option<DisplayInfo> = None;

        loop {
            // A `DisplayInfo` rather than the whole `Status`: this copy runs
            // with the `status` ceiling priority held, which masks the PPS edge
            // the transmission is aligned to, so it stays a few bytes wide.
            let info = cx.shared.status.lock(|status| status.display_info());

            // Redraw only on change, the snapshot above is deliberately outside the redraw
            if shown != Some(info) {
                if let Err(e) = display.show(&info) {
                    wspr_log!("DISP: show failed: {:?}", e);
                }
                shown = Some(info);
            }

            Mono::delay(CFG.sw.disp.poll_ms.millis()).await;
        }
    }
}
