#![no_main]
#![no_std]

#[cfg(feature = "rtt-log")]
use panic_rtt_target as _;

#[cfg(not(feature = "rtt-log"))]
use panic_halt as _;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [SPI1])]
mod app {
    use wspr_beacon::beacon::events::Event;
    use wspr_beacon::beacon::qth::{Coordinates, qth_square};
    use wspr_beacon::beacon::states::State;
    use wspr_beacon::{wspr_log, wspr_lognln};

    use cortex_m::singleton;
    use heapless::binary_heap::{BinaryHeap, Max};
    use nmea0183;
    use rtic_monotonics::stm32::prelude::*;
    use stm32f1xx_hal::{
        dma::CircBuffer,
        gpio::{Edge, ExtiPin, Input, Output, PushPull, gpiob::PB1, gpioc::PC13},
        pac::{self, DMA1, USART3},
        prelude::*,
        serial, timer,
    };
    use wspr_encoder;

    #[cfg(feature = "rtt-log")]
    use rtt_target::{rprint, rprintln, rtt_init_print};

    const SYSCLK_MHZ: u32 = 32;
    const UBLOX_LEN: usize = 2048;
    const CALLSIGN: &str = "R1BRL";

    stm32_tim4_monotonic!(Mono, 1_000_000);

    #[shared]
    struct Shared {
        state: State,
        queue: BinaryHeap<Event, Max, 4>,
        wspr_msg: Option<[u8; 162]>,
    }

    #[local]
    struct Local {
        // LED task
        led: PC13<Output<PushPull>>,
        tim3: timer::CounterMs<pac::TIM3>,
        // GPS task
        circ: Option<CircBuffer<[u8; UBLOX_LEN], serial::RxDma3>>,
        parser: nmea0183::Parser,
        // PPS task
        pps: PB1<Input>,
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

        rcc.clocks.sysclk().to_MHz();

        #[cfg(feature = "dwt-profile")]
        {
            cx.core.DCB.enable_trace();
            cx.core.DWT.enable_cycle_counter();
        }

        #[cfg(feature = "rtt-log")]
        rtt_init_print!();

        let mut afio = cx.device.AFIO.constrain(&mut rcc);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc);
        let mut gpioc = cx.device.GPIOC.split(&mut rcc);
        let channels = cx.device.DMA1.split(&mut rcc);

        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        //// LED
        let mut tim3 = cx.device.TIM3.counter_ms(&mut rcc);
        tim3.start(5000u32.millis()).unwrap();
        tim3.listen(timer::Event::Update);

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

        let state: State = State::GpsWait;
        let wspr_msg: Option<[u8; 162]> = None;
        let queue: BinaryHeap<Event, Max, 4> = BinaryHeap::new();

        //// Interrupts

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::USART3);
            pac::NVIC::unmask(pac::Interrupt::EXTI1);
        }

        Mono::start(SYSCLK_MHZ * 1_000_000);

        (
            Shared {
                state,
                queue,
                wspr_msg,
            },
            Local {
                // LED task
                led,
                tim3,
                // GPS task
                circ: Some(circ),
                parser: nmea_parser,
                // PPS task
                pps,
            },
        )
    }

    #[idle(shared = [state, queue, wspr_msg])]
    fn idle(mut cx: idle::Context) -> ! {
        let mut event: Event = Event::NIL;

        loop {
            cx.shared.queue.lock(|queue| {
                if !queue.is_empty()
                    && let Some(e) = queue.pop()
                {
                    event = e;
                }
            });

            match event {
                Event::PPS => {
                    #[cfg(feature = "dwt-profile")]
                    wspr_log!(
                        "Event PPS: DWT {} ms",
                        cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000
                    );
                }
                Event::GPS(_, _) => {
                    #[cfg(feature = "dwt-profile")]
                    wspr_log!(
                        "Event GPS: DWT {} ms",
                        cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000
                    );
                }
                Event::NOGPS => {
                    #[cfg(feature = "dwt-profile")]
                    wspr_log!(
                        "Event GPS: DWT {} ms",
                        cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000
                    );
                }
                _ => {}
            }

            cx.shared.state.lock(|state| match *state {
                State::GpsWait => match event {
                    Event::GPS((lat, lon), _) => {
                        cx.shared.wspr_msg.lock(|msg| {
                            wspr_log!("SCHED: GPS coords ({}, {})", lat as u8, lon as u8);
                            if msg.is_none() {
                                let coords = Coordinates {
                                    latitude: lat,
                                    longitude: lon,
                                };
                                let mut qth: [u8; 4] = [0, 0, 0, 0];
                                match qth_square(coords, &mut qth) {
                                    Ok(qth) => {
                                        wspr_log!("SCHED: calculated QTH {}", qth);
                                        match wspr_encoder::encode(CALLSIGN, qth, 37) {
                                            Ok(symbols) => {
                                                *msg = Some(symbols);
                                                *state = State::TxWait;
                                            }
                                            Err(e) => {
                                                wspr_log!(
                                                    "SCHED: fatal WSPR encoding failure: {:?}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        wspr_log!("SCHED: fatal QTH calculation failure: {:?}", e);
                                    }
                                }
                            } else {
                                *state = State::TxWait;
                            }
                        });
                    }
                    _ => {}
                },
                State::TxWait => match event {
                    Event::GPS(_, time) => {
                        wspr_log!("Event GPS: Time ({}:{}:{})", time.0, time.1, time.2 as u8);
                        if time.2 as u8 == 59u8 {
                            *state = State::TxReady;
                        }
                    }
                    Event::NOGPS => {
                        wspr_log!("SCHED: GPS lost in TxWait");
                        *state = State::GpsWait;
                    }
                    _ => {}
                },
                State::TxReady => match event {
                    Event::PPS => match wspr::spawn(42) {
                        Ok(_) => {
                            wspr_log!("SCHED: spawned WSPR");
                        }
                        Err(_) => {
                            wspr_log!("SCHED: failed to spawn WSPR")
                        }
                    },
                    Event::NOGPS => {
                        wspr_log!("SCHED: GPS lost in TxReady");
                        *state = State::GpsWait;
                    }
                    _ => {}
                },
                State::TxActive => {}
                State::TxDone => {
                    wspr_log!("SCHED: Tx completed");
                    cx.shared.wspr_msg.lock(|msg| {
                        *msg = None;
                    });
                    *state = State::GpsWait;
                }
                State::Error(code) => {
                    wspr_log!("SCHED: error code {}", code);
                    *state = State::GpsWait;
                }
            });

            event = Event::NIL;

            // Keep the core awake so host-side RTT attach does not time out.
            #[cfg(feature = "rtt-log")]
            cortex_m::asm::nop();

            #[cfg(not(feature = "rtt-log"))]
            cortex_m::asm::wfi();
        }
    }

    #[task(priority = 10, shared = [state, wspr_msg])]
    async fn wspr(mut cx: wspr::Context, x: i32) {
        let mut msg: Option<[u8; 162]> = None;
        let mut tx = false;

        wspr_log!("WSPR started: {}", x);

        cx.shared.state.lock(|state| {
            if *state == State::TxReady {
                *state = State::TxActive;
                tx = true;
            }
        });

        cx.shared.wspr_msg.lock(|wspr_msg| {
            msg = *wspr_msg;
        });

        if !tx {
            return;
        }

        match msg {
            Some(symbols) => {
                // 20m WSPR dial frequency in KHz
                let dial = 14095.6;

                // WSPR transmit frequencies are 1.5KHz above the dial frequency
                let offset = 1.5;

                for symbol in symbols.iter() {
                    let frequency = dial + offset + (0.001464 * (*symbol as f64));
                    // TODO
                    // set_frequency(frequency);
                    // enable_tx();
                    wspr_log!("WSPR: transmit symbol {} freq {}", symbol, frequency as u16);
                    Mono::delay(683_u64.millis()).await;
                    // disable_tx();
                }

                cx.shared.state.lock(|state| {
                    *state = State::TxDone;
                });
            }
            None => {
                cx.shared.state.lock(|state| {
                    *state = State::Error(10);
                });
            }
        }
    }

    #[task(binds = EXTI1, priority = 10, local = [pps], shared = [state, queue])]
    fn pps(mut cx: pps::Context) {
        if cx.local.pps.check_interrupt() {
            #[cfg(feature = "dwt-profile")]
            wspr_log!(
                "IRQ PPS: DWT {} ms",
                cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000
            );
            cx.local.pps.clear_interrupt_pending_bit();
            cx.shared.queue.lock(|queue| {
                queue.push(Event::PPS).ok();
            });
        }
    }

    #[task(binds = TIM3, priority = 1, local = [led, tim3], shared = [queue])]
    fn led(mut cx: led::Context) {
        cx.shared.queue.lock(|queue| {
            queue.push(Event::LED).ok();
        });

        cx.local.tim3.clear_interrupt(timer::Event::Update);
        cx.local.led.toggle();
    }

    #[task(binds = USART3, priority = 5, local = [circ, parser], shared = [state, queue])]
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

                #[cfg(feature = "rtt-log-verbose")]
                {
                    let recv = (UBLOX_LEN * 2) - unsafe { (*DMA1::ptr()).ch3().ndtr().read().ndt().bits() as usize };
                    buf[0][..recv].iter().for_each(|&b| wspr_lognln!("{}", b as char));
                }

                cx.shared.state.lock(|state| {
                    if *state == State::TxActive {
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
                            queue.push(Event::GPS((lat, lon), (h, m, s))).ok();
                        } else {
                            queue.push(Event::NOGPS).ok();
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
}
