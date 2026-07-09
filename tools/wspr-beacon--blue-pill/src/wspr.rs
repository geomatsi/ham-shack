#![no_main]
#![no_std]

use core::cmp::Ordering;
use panic_rtt_target as _;

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    WaitGps,
    TxWait,
    TxReady,
    TxActive,
    TxDone,
}

#[derive(Clone, Copy, Default)]
pub enum Event {
    /// Empty
    #[default]
    NIL,
    /// LED data
    LED,
    /// No GPS fix
    NOGPS,
    /// GPS data
    GPS((u8, u8), (u8, u8), (u8, u8, f32)),
    /// PPS data
    PPS,
}

impl Event {
    fn prio(self) -> u8 {
        match self {
            Event::NIL => 0u8,
            Event::LED => 10u8,
            Event::NOGPS => 20u8,
            Event::GPS(_, _, _) => 20u8,
            Event::PPS => 50u8,
        }
    }
}

/* simple ordering of events based only on their priority */

impl Eq for Event {}

impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.prio() == other.prio()
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        match self {
            Event::NIL => match other {
                Event::NIL => Ordering::Equal,
                _ => self.prio().cmp(&other.prio()),
            },
            Event::LED => match other {
                Event::LED => Ordering::Equal,
                _ => self.prio().cmp(&other.prio()),
            },
            Event::NOGPS => match other {
                Event::NOGPS => Ordering::Equal,
                _ => self.prio().cmp(&other.prio()),
            },
            Event::GPS(_, _, _) => match other {
                Event::GPS(_, _, _) => Ordering::Equal,
                _ => self.prio().cmp(&other.prio()),
            },
            Event::PPS => match other {
                Event::PPS => Ordering::Equal,
                _ => self.prio().cmp(&other.prio()),
            },
        }
    }
}

////

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [SPI1])]
mod app {
    use crate::Event;
    use crate::State;

    use cortex_m::singleton;
    use heapless::binary_heap::{BinaryHeap, Max};
    use nmea0183;
    use rtic_monotonics::stm32::prelude::*;
    use rtt_target::{rprintln, rtt_init_print};
    use stm32f1xx_hal::{
        dma::CircBuffer,
        gpio::{Edge, ExtiPin, Input, Output, PushPull, gpiob::PB1, gpioc::PC13},
        pac::{self, DMA1, USART3},
        prelude::*,
        serial, timer,
    };
    use wspr_encoder;

    const SYSCLK_MHZ: u32 = 32;
    const UBLOX_LEN: usize = 2048;

    stm32_tim4_monotonic!(Mono, 1_000_000);

    #[shared]
    struct Shared {
        state: State,
        queue: BinaryHeap<Event, Max, 4>,
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
        let mut cp = cx.core;

        rcc.clocks.sysclk().to_MHz();

        cp.DCB.enable_trace();
        cp.DWT.enable_cycle_counter();

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

        //// Interrupts

        let state: State = State::WaitGps;
        let queue: BinaryHeap<Event, Max, 4> = BinaryHeap::new();

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::USART3);
            pac::NVIC::unmask(pac::Interrupt::EXTI1);
        }

        Mono::start(SYSCLK_MHZ * 1_000_000);

        (
            Shared { state, queue },
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

    #[idle(shared = [state, queue])]
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
                    rprintln!("Event PPS (DWT {})", cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000);
                }
                Event::GPS(lat, lon, time) => {
                    rprintln!(
                        "Event GPS: LOC ({}, {}) TIME ({}:{}:{}) (DWT {})",
                        lat.0,
                        lon.0,
                        time.0,
                        time.1,
                        time.2 as u8,
                        cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000
                    );
                }
                Event::NOGPS => {
                    rprintln!("Event GPS: no fix");
                }
                _ => {}
            }

            cx.shared.state.lock(|state| match *state {
                State::WaitGps => match event {
                    Event::GPS(_, _, _) => {
                        *state = State::TxWait;
                    }
                    _ => {}
                },
                State::TxWait => match event {
                    Event::GPS(_, _, time) => {
                        if time.2 as u8 == 59u8 {
                            *state = State::TxReady;
                        }
                    }
                    Event::NOGPS => {
                        *state = State::WaitGps;
                    }
                    _ => {}
                },
                State::TxReady => match event {
                    Event::PPS => match wspr::spawn(42) {
                        Ok(_) => {
                            rprintln!("PPS: spawned WSPR");
                        }
                        Err(_) => {
                            rprintln!("PPS: WSPR is already running")
                        }
                    },
                    Event::NOGPS => {
                        *state = State::WaitGps;
                    }
                    _ => {}
                },
                State::TxActive => {}
                State::TxDone => {
                    *state = State::TxWait;
                }
            });

            event = Event::NIL;

            // Keep the core awake so host-side RTT attach does not time out.
            cortex_m::asm::nop();
        }
    }

    #[task(priority = 10, shared = [state])]
    async fn wspr(mut cx: wspr::Context, x: i32) {
        let mut tx = false;
        rprintln!("WSPR started: {}", x);

        cx.shared.state.lock(|state| {
            if *state == State::TxReady {
                *state = State::TxActive;
                tx = true;
            }
        });

        if !tx {
            return;
        }

        match wspr_encoder::encode("R1BRL", "KP50", 37) {
            Ok(symbols) => {
                // 20m WSPR dial frequency in KHz
                let dial = 14095.6;

                // WSPR transmit frequencies are 1.5KHz above the dial frequency
                let offset = 1.5;

                for symbol in symbols.iter() {
                    let _frequency = dial + offset + (0.001464 * (*symbol as f64));
                    // TODO
                    // set_frequency(frequency);
                    // enable_tx();
                    Mono::delay(50_u64.millis() /* 683_u64.millis() */).await;
                    // disable_tx();
                }
            }
            Err(e) => {
                rprintln!("WSPR: fatal encoding failure: {:?}", e);
            }
        }

        cx.shared.state.lock(|state| {
            *state = State::TxDone;
        });
    }

    #[task(binds = EXTI1, priority = 10, local = [pps], shared = [state, queue])]
    fn pps(mut cx: pps::Context) {
        if cx.local.pps.check_interrupt() {
            rprintln!("IRQ PPS (DWT {})", cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000);
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

            rprintln!("IRQ GPS (DWT {})", cortex_m::peripheral::DWT::cycle_count() / SYSCLK_MHZ / 1_000);

            if let Some(circ) = cx.local.circ.take() {
                let (buf, rxdma) = circ.stop();
                let _recv = (UBLOX_LEN * 2)
                    - unsafe { (*DMA1::ptr()).ch3().ndtr().read().ndt().bits() as usize };

                cx.shared.state.lock(|state| {
                    if *state == State::TxActive {
                        process_nmea = false;
                    }
                });

                //let start = cortex_m::peripheral::DWT::cycle_count();

                if process_nmea {
                    let mut fix = false;
                    let mut latd: u8 = 0u8;
                    let mut latm: u8 = 0u8;
                    let mut lond: u8 = 0u8;
                    let mut lonm: u8 = 0u8;
                    let mut h = 0;
                    let mut m = 0;
                    let mut s = 0f32;

                    for result in cx.local.parser.parse_from_bytes(&buf[0][..]) {
                        match result {
                            Ok(nmea0183::ParseResult::RMC(Some(rmc))) => {
                                h = rmc.datetime.time.hours;
                                m = rmc.datetime.time.minutes;
                                s = rmc.datetime.time.seconds;
                                latd = rmc.latitude.degrees;
                                latm = rmc.latitude.minutes;
                                lond = rmc.longitude.degrees;
                                lonm = rmc.longitude.minutes;
                                fix = true;
                            }
                            Ok(nmea0183::ParseResult::RMC(None)) => {
                                fix = false;
                            }
                            Ok(_) => {
                                // skip other messages for now
                            }
                            Err(e) => {
                                rprintln!("Error parsing NMEA: {}", e);
                            }
                        }
                    }

                    cx.shared.queue.lock(|queue| {
                        if fix {
                            queue
                                .push(Event::GPS((latd, latm), (lond, lonm), (h, m, s)))
                                .ok();
                        } else {
                            queue.push(Event::NOGPS).ok();
                        }
                    });
                }

                buf[0].fill(0);
                //rprintln!("NMEA processing: {} us", cortex_m::peripheral::DWT::cycle_count().wrapping_sub(start) / SYSCLK_MHZ);

                *cx.local.circ = Some(rxdma.circ_read(buf));
            }
        }
    }
}
