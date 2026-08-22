#![no_main]
#![no_std]

#[cfg(feature = "rtt-log")]
use panic_rtt_target as _;

#[cfg(not(feature = "rtt-log"))]
use panic_halt as _;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [SPI1, SPI2])]
mod app {
    use core::sync::atomic::{AtomicBool, Ordering, compiler_fence};
    use stm32f1xx_hal::i2c::BlockingI2c;
    use wspr_beacon::beacon::calibration::{Calibration, Reading, Step};
    use wspr_beacon::beacon::events::Event;
    use wspr_beacon::beacon::qth::{Coordinates, qth_square};
    use wspr_beacon::beacon::states::{ErrorState, State};
    use wspr_beacon::beacon::status::{DisplayInfo, Status, Time};
    use wspr_beacon::wspr_log;

    use wspr_beacon::hw::display::{self, StatusDisplay};

    #[cfg(feature = "rtt-log-debug")]
    use wspr_beacon::wspr_lognln;

    use cortex_m::singleton;
    use heapless::binary_heap::{BinaryHeap, Max};
    use nmea0183;
    use rtic_monotonics::stm32::prelude::*;
    use si5351::{ClockOutput, DriveStrength, Frequency, PLL, Si5351, Si5351Device, calibrate};
    use stm32f1xx_hal::{
        dma::CircBuffer,
        gpio::{self, Edge, ExtiPin, Input},
        i2c::{DutyCycle, Mode},
        pac::{self, USART3},
        prelude::*,
        serial,
        timer::Timer,
    };
    use wspr_encoder;

    // DMA1 is only touched by the rtt-log-debug byte dump in the GPS handler.
    #[cfg(feature = "rtt-log-debug")]
    use stm32f1xx_hal::pac::DMA1;

    #[cfg(feature = "rtt-log-debug")]
    use rtt_target::rprint;
    #[cfg(feature = "rtt-log")]
    use rtt_target::{rprintln, rtt_init_print};

    static PPS_WSPR_XMIT_GATE: AtomicBool = AtomicBool::new(false);
    static CALIB_PPS_EVT_GATE: AtomicBool = AtomicBool::new(false);

    const SYSCLK_MHZ: u32 = 32;
    const UBLOX_LEN: usize = 2048;
    const CALLSIGN: &str = "R1BRL";
    const PLL_PARKED: Frequency = Frequency::from_hz(62 * 14_097_100);
    const CLK1_FREQ: u32 = 10_000_000;
    const NOMINAL: Frequency = Frequency::from_hz(CLK1_FREQ);

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
        circ: Option<CircBuffer<[u8; UBLOX_LEN], serial::RxDma3>>,
        parser: nmea0183::Parser,

        // PPS task
        pps: gpio::gpiob::PB1<Input>,

        // Display task
        display: Option<display::Display>,

        // PPS task
        tim2: pac::TIM2,
        tim3: pac::TIM3,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local) {
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.freeze(
            stm32f1xx_hal::rcc::Config::hse(8.MHz())
                .sysclk(SYSCLK_MHZ.MHz())
                .pclk1(16.MHz()),
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

        //// Si5351 generator
        let scl = gpiob.pb8;
        let sda = gpiob.pb9;

        let i2c = cx.device.I2C1.remap(&mut afio.mapr).blocking_i2c(
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
        xmit.select_clock_pll(ClockOutput::Clk1, PLL::A);
        xmit.set_clock_drive(ClockOutput::Clk0, DriveStrength::_8mA);
        xmit.set_clock_drive(ClockOutput::Clk1, DriveStrength::_8mA);
        xmit.set_clock_enabled(ClockOutput::Clk0, false);
        xmit.set_clock_enabled(ClockOutput::Clk1, false);
        xmit.set_clock_enabled(ClockOutput::Clk2, false);
        xmit.flush_output_enabled().unwrap();
        xmit.set_pll_frequency(PLL::A, PLL_PARKED).unwrap();
        xmit.reset_pll(PLL::A).unwrap();

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
            .serial((stx, srx), 9600.bps(), &mut rcc)
            .split();

        // setup serial 'idle' interrupt before converting rx into rxdma
        rx.listen_idle();

        let nmea_parser = nmea0183::Parser::new()
            .sentence_filter(nmea0183::Sentence::RMC | nmea0183::Sentence::GGA);
        let dmabuf = singleton!(: [[u8; UBLOX_LEN]; 2] = [[0; UBLOX_LEN]; 2]).unwrap();
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
            },
        )
    }

    #[idle(shared = [status, queue, xmit])]
    fn idle(mut cx: idle::Context) -> ! {
        let mut calib: Option<Calibration> = None;
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
                #[cfg(feature = "dwt-profile")]
                match event {
                    Event::GPS(_, _) => {
                        wspr_log!(
                            "Event GPS: DWT {} ms",
                            cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000
                        );
                    }
                    Event::NOGPS => {
                        wspr_log!(
                            "Event NOGPS: DWT {} ms",
                            cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000
                        );
                    }
                    _ => {}
                }

                // Event handling: fill displayed information
                // - QTH is not maintained here: it is filled in the FSM where WSPR is encoded
                if let Event::GPS(_, (hours, minutes, _)) = event {
                    cx.shared.status.lock(|status| {
                        status.time = Some(Time { hours, minutes });
                    });
                }

                // Event handling: main FSM
                cx.shared.status.lock(|status| match status.state {
                    State::GpsWait => match event {
                        Event::GPS((lat, lon), _) => {
                            wspr_log!("SCHED: GPS coords ({}, {})", lat as u8, lon as u8);

                            if status.msg.is_some() {
                                status.state = State::TxWait;
                            } else {
                                let coords = Coordinates {
                                    latitude: lat,
                                    longitude: lon,
                                };
                                let mut qth_buf: [u8; 4] = [0, 0, 0, 0];
                                #[cfg(feature = "dwt-profile")]
                                let qth_start = cortex_m::peripheral::DWT::cycle_count();

                                let encoded = match qth_square(coords, &mut qth_buf) {
                                    Ok(qth) => {
                                        #[cfg(feature = "dwt-profile")]
                                        wspr_log!(
                                            "SCHED: QTH calculation: {} us",
                                            cortex_m::peripheral::DWT::cycle_count()
                                                .wrapping_sub(qth_start)
                                                / SYSCLK_MHZ
                                        );

                                        wspr_log!("SCHED: calculated QTH {}", qth);

                                        #[cfg(feature = "dwt-profile")]
                                        let enc_start = cortex_m::peripheral::DWT::cycle_count();

                                        match wspr_encoder::encode(CALLSIGN, qth, 37) {
                                            Ok(symbols) => {
                                                #[cfg(feature = "dwt-profile")]
                                                wspr_log!(
                                                    "SCHED: WSPR encoding: {} us",
                                                    cortex_m::peripheral::DWT::cycle_count()
                                                        .wrapping_sub(enc_start)
                                                        / SYSCLK_MHZ
                                                );

                                                status.msg = Some(symbols);
                                                status.state = State::TxWait;

                                                true
                                            }
                                            Err(e) => {
                                                wspr_log!(
                                                    "SCHED: fatal WSPR encoding failure: {:?}",
                                                    e
                                                );

                                                false
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        wspr_log!("SCHED: fatal QTH calculation failure: {:?}", e);

                                        false
                                    }
                                };

                                // Display QTH only if the WSPR encoding succeeded. It is
                                // published here rather than next to `status.msg` above
                                // because `qth` borrows `qth_buf` until the match ends.
                                if encoded {
                                    status.qth = Some(qth_buf);
                                }
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
                        Event::GPS(_, time) => {
                            wspr_log!("SCHED: TxWait: GPS: Time ({}:{}:{})", time.0, time.1, time.2 as u8);
                            // 30 sec before WSPR slot: start calibraton task to tune si5351
                            // TODO: write proper time condition for 1 min before WSPR slot
                            if time.2 as u8 == 10u8 {
                                wspr_log!("SCHED: started calibration");
                                CALIB_PPS_EVT_GATE.store(true, Ordering::Release);
                                calib = Some(Calibration::new(NOMINAL));
                                status.state = State::TxCalib;
                            }
                            // 1 sec before WSPR slot: spawn WSPR task and move to TxActive state
                            // TODO: write proper time condition for 1 sec before WSPR slot
                            if time.2 as u8 == 59u8 && status.ppb.is_some() {
                                match wspr::spawn(42) {
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
                    State::TxCalib => {
                        if let Event::CALIB(ticks) = event {
                            if let Some(c) = calib.as_mut() {
                                let step = c.gate(ticks);

                                match c.last_reading() {
                                    Reading::Ppb(ppb) => {
                                        wspr_log!(
                                            "SCHED: calibraton: {} ticks ({:+} vs nominal) ppb {}",
                                            ticks,
                                            ticks as i64 - CLK1_FREQ as i64,
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
                                        cx.shared.xmit.lock(|xmit| {
                                            xmit.set_clock_frequency_fixed_pll(ClockOutput::Clk1, dial)
                                                .unwrap();
                                        });
                                    }
                                    Step::Stop(result) => {
                                        cx.shared.xmit.lock(|xmit| {
                                            xmit.set_clock_enabled(ClockOutput::Clk1, false);
                                            xmit.flush_output_enabled().unwrap();
                                        });

                                        match result {
                                            Some(ppb) => {
                                                wspr_log!("CALIB: completed calibration with {} ppb", ppb)
                                            }
                                            None => {
                                                wspr_log!("CALIB: calibration failed: no usable gate")
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
                                wspr_log!("CALIB: unexpected state, stop and reset calibration");
                                cx.shared.xmit.lock(|xmit| {
                                    xmit.set_clock_enabled(ClockOutput::Clk1, false);
                                    xmit.flush_output_enabled().unwrap();
                                });
                                CALIB_PPS_EVT_GATE.store(false, Ordering::Release);
                                status.state = State::TxWait;
                                status.ppb = None;
                            }
                        }
                    }
                    State::TxActive => {
                        if let Event::TXDONE = event {
                            // Start over from a fresh fix before the next transmission
                            wspr_log!("SCHED: Tx completed");
                            status.reset();
                        }
                    }
                    State::Error(code) => {
                        wspr_log!("SCHED: error code {}", code);
                        status.reset();
                        // TODO
                        // - add last error state to Status and display it for diagnostic purposes
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

    #[task(priority = 10, shared = [queue, status, xmit])]
    async fn wspr(mut cx: wspr::Context, x: i32) {
        wspr_log!("WSPR started: {}", x);

        // spawned after 59th second, so gate on the next PPS
        // TODO: handle lost GPS and missing PPS using Mono (?)
        PPS_WSPR_XMIT_GATE.store(true, Ordering::Relaxed);
        while PPS_WSPR_XMIT_GATE.load(Ordering::Acquire) {
            Mono::delay(1u64.millis()).await;
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
            // WSPR dial frequency for 20m band
            let dial: Frequency = Frequency::from_hz(14_095_600);
            // WSPR audio offset is 1.5KHz above the dial frequency
            let offset = Frequency::from_hz(1_500);

            let tx_start = Mono::now();

            for (num, symbol) in symbols.iter().enumerate() {
                // Absolute deadline for the end of this symbol, computed from
                // tx_start with integer math. Deriving each deadline straight from
                // tx_start — rather than summing a rounded per-symbol duration —
                // keeps the error per boundary instead of letting it accumulate:
                // a rounded 683 ms period would drift ~54 ms by symbol 161.
                let elapsed_us =
                    WSPR_SYMBOL_SAMPLES * (num as u64 + 1) * 1_000_000u64 / WSPR_SAMPLE_RATE_HZ;
                let deadline = tx_start + elapsed_us.micros();

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

                cx.shared.xmit.lock(|xmit| {
                    xmit.set_clock_frequency_fixed_pll(ClockOutput::Clk0, freq)
                        .unwrap();
                });

                Mono::delay_until(deadline).await;
            }
        } else {
            wspr_log!("WSPR: empty wspr message or empty ppb not expected");
        }

        cx.shared.xmit.lock(|xmit| {
            xmit.set_clock_enabled(ClockOutput::Clk0, false);
            xmit.flush_output_enabled().unwrap();
        });

        cx.shared.queue.lock(|queue| {
            if queue.push(Event::TXDONE).is_err() {
                wspr_log!("WSPR: failed to send TXDONE event");
                // emergency exit from TxActive state
                cx.shared.status.lock(|status| {
                    status.state = State::Error(ErrorState::WSPRQueueFailure);
                });
            }
        });
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
                                status.state = State::Error(ErrorState::CALIBQueueFailure);
                            });
                        }
                    });
                }
            } else {
                enable();
            }

            #[cfg(feature = "dwt-profile")]
            wspr_log!(
                "IRQ PPS: DWT {} ms",
                cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000
            );
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

            #[cfg(feature = "dwt-profile")]
            wspr_log!(
                "IRQ GPS: DWT {} ms",
                cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000
            );

            if let Some(circ) = cx.local.circ.take() {
                let (buf, rxdma) = circ.stop();

                #[cfg(feature = "rtt-log-debug")]
                {
                    let recv = (UBLOX_LEN * 2)
                        - unsafe { (*DMA1::ptr()).ch3().ndtr().read().ndt().bits() as usize };
                    buf[0][..recv.min(UBLOX_LEN)]
                        .iter()
                        .for_each(|&b| wspr_lognln!("{}", b as char));
                }

                cx.shared.status.lock(|status| {
                    if matches!(status.state, State::TxActive | State::TxCalib) {
                        process_nmea = false;
                    }
                });

                if process_nmea {
                    #[cfg(feature = "dwt-profile")]
                    let start = cortex_m::peripheral::DWT::cycle_count();

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

                    #[cfg(feature = "dwt-profile")]
                    wspr_log!(
                        "IRQ GPS: NMEA processing: {} us",
                        cortex_m::peripheral::DWT::cycle_count().wrapping_sub(start) / SYSCLK_MHZ
                    );
                }

                buf[0].fill(0);
                *cx.local.circ = Some(rxdma.circ_read(buf));
            }
        }
    }

    #[task(priority = 1, shared = [status], local = [display])]
    async fn display_task(mut cx: display_task::Context) {
        const POLL_MS: u64 = 250;

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

            Mono::delay(POLL_MS.millis()).await;
        }
    }
}
