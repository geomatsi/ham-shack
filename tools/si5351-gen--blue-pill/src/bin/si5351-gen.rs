#![no_main]
#![no_std]

use bitbang_hal::i2c::I2cBB;
use cortex_m as cm;
use cortex_m_rt::entry;
use hal::prelude::*;
use hd44780_driver::bus::DataBus;
use hd44780_driver::{CursorBlink, Display, DisplayMode, HD44780};
use panic_semihosting as _;
use rotary_encoder_embedded::{Direction, RotaryEncoder};
use stm32_gen::support::bitbang_i2c_compat::Eh1BitBangI2c;
use stm32_gen::support::si5351::{ClockOutput, PLL, Si5351, Si5351Device};
use stm32f1xx_hal as hal;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::rcc;

const LINE_WIDTH: usize = 16;
const TITLE_LINE_POS: u8 = 0x00;
const STATUS_LINE_POS: u8 = 0x40;

const BUTTON_DEBOUNCE_TICKS: u8 = 20;
const NORMAL_PRESS_TICKS: u16 = 200;
const LONG_PRESS_TICKS: u16 = NORMAL_PRESS_TICKS * 3;
const DOUBLE_SHORT_PRESS_TICKS: u16 = NORMAL_PRESS_TICKS * 2;

const FREQ_MIN_HZ: u32 = 4_000;
const FREQ_MAX_HZ: u32 = 150_000_000;

const STEP_VALUES: [u32; 9] = [
    10, 100, 500, 1_000, 10_000, 100_000, 500_000, 1_000_000, 10_000_000,
];

#[derive(Copy, Clone, Eq, PartialEq)]
enum Control {
    Frequency,
    Step,
    State,
}

impl Control {
    fn next(self) -> Self {
        match self {
            Self::Frequency => Self::Step,
            Self::Step => Self::State,
            Self::State => Self::Frequency,
        }
    }
}

struct ChannelState {
    name: &'static [u8],
    pll: PLL,
    output: ClockOutput,
    applied_freq: u32,
    draft_freq: u32,
    applied_step_idx: usize,
    draft_step_idx: usize,
    applied_enabled: bool,
    draft_enabled: bool,
}

impl ChannelState {
    const fn new(name: &'static [u8], pll: PLL, output: ClockOutput, freq: u32) -> Self {
        Self {
            name,
            pll,
            output,
            applied_freq: freq,
            draft_freq: freq,
            applied_step_idx: 0,
            draft_step_idx: 0,
            applied_enabled: false,
            draft_enabled: false,
        }
    }

    fn has_pending_changes(&self) -> bool {
        self.applied_freq != self.draft_freq
            || self.applied_step_idx != self.draft_step_idx
            || self.applied_enabled != self.draft_enabled
    }
}

fn format_u32(mut value: u32, buffer: &mut [u8; 10]) -> &[u8] {
    let mut index = buffer.len();

    loop {
        index -= 1;
        buffer[index] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }

    &buffer[index..]
}

fn format_freq_field(freq: u32, buffer: &mut [u8; 10]) -> &[u8] {
    let (divisor, decimals, suffix) = if freq >= 100_000_000 {
        (1_000_000, 1usize, b'M')
    } else if freq >= 10_000_000 {
        (1_000_000, 2usize, b'M')
    } else if freq >= 1_000_000 {
        (1_000_000, 3usize, b'M')
    } else if freq >= 100_000 {
        (1_000, 1usize, b'K')
    } else if freq >= 10_000 {
        (1_000, 2usize, b'K')
    } else if freq >= 1_000 {
        (1_000, 3usize, b'K')
    } else {
        (1, 0usize, b'H')
    };

    let whole = freq / divisor;
    let mut whole_digits = [0u8; 10];
    let whole_field = format_u32(whole, &mut whole_digits);
    let mut index = 0usize;

    for &byte in whole_field {
        buffer[index] = byte;
        index += 1;
    }

    if decimals > 0 {
        buffer[index] = b'.';
        index += 1;

        let denom = match decimals {
            1 => divisor / 10,
            2 => divisor / 100,
            _ => divisor / 1_000,
        };
        let mut fraction = (freq % divisor) / denom;

        let mut digits = [b'0'; 3];
        for slot in (0..decimals).rev() {
            digits[slot] = b'0' + (fraction % 10) as u8;
            fraction /= 10;
        }

        for &digit in &digits[..decimals] {
            buffer[index] = digit;
            index += 1;
        }
    }

    buffer[index] = suffix;
    index += 1;
    &buffer[..index]
}

fn step_label(step_idx: usize) -> &'static [u8] {
    match STEP_VALUES[step_idx] {
        10 => b"10H",
        100 => b"100H",
        500 => b"500H",
        1_000 => b"1K",
        10_000 => b"10K",
        100_000 => b"100K",
        500_000 => b"500K",
        1_000_000 => b"1M",
        10_000_000 => b"10M",
        _ => b"?",
    }
}

fn state_label(enabled: bool) -> &'static [u8] {
    if enabled { b"ON " } else { b"OFF" }
}

fn write_line<B, D>(lcd: &mut HD44780<B>, delay: &mut D, pos: u8, line: &[u8; LINE_WIDTH])
where
    B: DataBus,
    D: embedded_hal::blocking::delay::DelayUs<u16> + embedded_hal::blocking::delay::DelayMs<u8>,
{
    lcd.set_cursor_pos(pos, delay).unwrap();
    lcd.write_bytes(line, delay).unwrap();
}

fn write_title_line<B, D>(lcd: &mut HD44780<B>, delay: &mut D, channel: &ChannelState)
where
    B: DataBus,
    D: embedded_hal::blocking::delay::DelayUs<u16> + embedded_hal::blocking::delay::DelayMs<u8>,
{
    let mut line = [b' '; LINE_WIDTH];
    let width = channel.name.len().min(line.len());
    line[..width].copy_from_slice(&channel.name[..width]);
    if channel.has_pending_changes() && width < LINE_WIDTH {
        line[width] = b'*';
    }
    write_line(lcd, delay, TITLE_LINE_POS, &line);
}

fn write_status_line<B, D>(
    lcd: &mut HD44780<B>,
    delay: &mut D,
    channel: &ChannelState,
    selected_control: Control,
) where
    B: DataBus,
    D: embedded_hal::blocking::delay::DelayUs<u16> + embedded_hal::blocking::delay::DelayMs<u8>,
{
    let mut line = [b' '; LINE_WIDTH];
    let mut freq = [0u8; 10];
    let freq_field = format_freq_field(channel.draft_freq, &mut freq);
    let freq_width = freq_field.len().min(6);

    line[0] = if selected_control == Control::Frequency {
        b'>'
    } else {
        b' '
    };
    line[1..1 + freq_width].copy_from_slice(&freq_field[..freq_width]);

    line[7] = if selected_control == Control::Step {
        b'>'
    } else {
        b' '
    };
    let step_field = step_label(channel.draft_step_idx);
    let step_width = step_field.len().min(4);
    line[8..8 + step_width].copy_from_slice(&step_field[..step_width]);

    line[12] = if selected_control == Control::State {
        b'>'
    } else {
        b' '
    };
    let state_field = state_label(channel.draft_enabled);
    line[13..16].copy_from_slice(state_field);

    write_line(lcd, delay, STATUS_LINE_POS, &line);
}

fn adjust_frequency(freq: &mut u32, step: u32, direction: Direction) {
    match direction {
        Direction::Clockwise => {
            *freq = freq.saturating_add(step).min(FREQ_MAX_HZ);
        }
        Direction::Anticlockwise => {
            *freq = freq.saturating_sub(step).max(FREQ_MIN_HZ);
        }
        Direction::None => {}
    }
}

fn apply_channel_output<I>(device: &mut Si5351Device<I>, channel: &ChannelState)
where
    I: embedded_hal_1::i2c::I2c,
{
    device
        .set_frequency(channel.pll, channel.output, channel.applied_freq)
        .unwrap();
    device.set_clock_enabled(channel.output, channel.applied_enabled);
    device.flush_clock_control(channel.output).unwrap();
    device.flush_output_enabled().unwrap();
}

fn apply_selected_control<I>(
    device: &mut Si5351Device<I>,
    channel: &mut ChannelState,
    selected_control: Control,
) where
    I: embedded_hal_1::i2c::I2c,
{
    match selected_control {
        Control::Frequency => {
            channel.applied_freq = channel.draft_freq;
            apply_channel_output(device, channel);
        }
        Control::Step => {
            channel.applied_step_idx = channel.draft_step_idx;
        }
        Control::State => {
            channel.applied_enabled = channel.draft_enabled;
            device.set_clock_enabled(channel.output, channel.applied_enabled);
            device.flush_clock_control(channel.output).unwrap();
            device.flush_output_enabled().unwrap();
        }
    }
}

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

    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);
    let mut delay = cp.SYST.delay(&rcc.clocks);

    let (_pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    let rs = gpiob.pb9.into_push_pull_output(&mut gpiob.crh);
    let mut rw = gpiob.pb8.into_push_pull_output(&mut gpiob.crh);
    let en = gpiob.pb7.into_push_pull_output(&mut gpiob.crl);

    rw.set_low();

    let mut b0 = pb3.into_push_pull_output(&mut gpiob.crl);
    let mut b1 = pb4.into_push_pull_output(&mut gpiob.crl);
    let mut b2 = gpiob.pb5.into_push_pull_output(&mut gpiob.crl);
    let mut b3 = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);

    b0.set_low();
    b1.set_low();
    b2.set_low();
    b3.set_low();

    let b4 = gpioa.pa9.into_push_pull_output(&mut gpioa.crh);
    let b5 = gpioa.pa10.into_push_pull_output(&mut gpioa.crh);
    let b6 = gpioa.pa11.into_push_pull_output(&mut gpioa.crh);
    let b7 = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);

    let mut lcd = HD44780::new_4bit(rs, en, b4, b5, b6, b7, &mut delay).unwrap();
    lcd.clear(&mut delay).unwrap();
    lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: hd44780_driver::Cursor::Invisible,
            cursor_blink: CursorBlink::Off,
        },
        &mut delay,
    )
    .unwrap();

    let button = gpioa.pa2.into_pull_up_input(&mut gpioa.crl);
    let rotary_dt = gpioa.pa1.into_pull_up_input(&mut gpioa.crl);
    let rotary_clk = gpioa.pa0.into_pull_up_input(&mut gpioa.crl);
    let mut encoder = RotaryEncoder::new(rotary_dt, rotary_clk).into_standard_mode();

    let scl = gpiob.pb12.into_open_drain_output(&mut gpiob.crh);
    let sda = gpiob.pb13.into_open_drain_output(&mut gpiob.crh);
    let mut i2c_timer = dp.TIM2.counter_hz(&mut rcc);
    i2c_timer.start(200.kHz()).unwrap();
    let i2c = I2cBB::new(scl, sda, i2c_timer);
    let i2c = Eh1BitBangI2c::new(i2c);

    let mut clock = Si5351Device::new_adafruit_module(i2c);
    clock.init_adafruit_module().unwrap();

    let mut channels = [
        ChannelState::new(b"CLK0", PLL::A, ClockOutput::Clk0, 7_000_000),
        ChannelState::new(b"CLK1", PLL::B, ClockOutput::Clk1, 14_000_000),
    ];

    for channel in &channels {
        apply_channel_output(&mut clock, channel);
    }

    let mut active_channel = 0usize;
    let mut selected_control = Control::Frequency;
    let mut title_dirty = true;
    let mut status_dirty = true;

    let mut button_pressed = button.is_low();
    let mut button_candidate = button_pressed;
    let mut button_stable_ticks = 0u8;
    let mut button_press_ticks = 0u16;
    let mut pending_short_press = false;
    let mut short_press_gap_ticks = 0u16;

    loop {
        {
            let channel = &mut channels[active_channel];
            match encoder.update() {
                Direction::Clockwise => {
                    match selected_control {
                        Control::Frequency => adjust_frequency(
                            &mut channel.draft_freq,
                            STEP_VALUES[channel.draft_step_idx],
                            Direction::Clockwise,
                        ),
                        Control::Step => {
                            channel.draft_step_idx =
                                (channel.draft_step_idx + 1) % STEP_VALUES.len();
                        }
                        Control::State => {
                            channel.draft_enabled = !channel.draft_enabled;
                        }
                    }
                    title_dirty = true;
                    status_dirty = true;
                }
                Direction::Anticlockwise => {
                    match selected_control {
                        Control::Frequency => adjust_frequency(
                            &mut channel.draft_freq,
                            STEP_VALUES[channel.draft_step_idx],
                            Direction::Anticlockwise,
                        ),
                        Control::Step => {
                            channel.draft_step_idx = if channel.draft_step_idx == 0 {
                                STEP_VALUES.len() - 1
                            } else {
                                channel.draft_step_idx - 1
                            };
                        }
                        Control::State => {
                            channel.draft_enabled = !channel.draft_enabled;
                        }
                    }
                    title_dirty = true;
                    status_dirty = true;
                }
                Direction::None => {}
            }
        }

        let sampled_pressed = button.is_low();
        if sampled_pressed == button_candidate {
            if button_stable_ticks < BUTTON_DEBOUNCE_TICKS {
                button_stable_ticks += 1;
            }
        } else {
            button_candidate = sampled_pressed;
            button_stable_ticks = 0;
        }

        if button_stable_ticks >= BUTTON_DEBOUNCE_TICKS && button_candidate != button_pressed {
            button_pressed = button_candidate;

            if button_pressed {
                button_press_ticks = 0;
            } else if button_press_ticks >= LONG_PRESS_TICKS {
                pending_short_press = false;
                short_press_gap_ticks = 0;
                active_channel = (active_channel + 1) % channels.len();
                title_dirty = true;
                status_dirty = true;
            } else {
                if pending_short_press && short_press_gap_ticks < DOUBLE_SHORT_PRESS_TICKS {
                    apply_selected_control(
                        &mut clock,
                        &mut channels[active_channel],
                        selected_control,
                    );
                    pending_short_press = false;
                    short_press_gap_ticks = 0;
                    title_dirty = true;
                    status_dirty = true;
                } else {
                    pending_short_press = true;
                    short_press_gap_ticks = 0;
                }
            }
        }

        if button_pressed && button_press_ticks < LONG_PRESS_TICKS {
            button_press_ticks += 1;
        }

        if pending_short_press && short_press_gap_ticks < DOUBLE_SHORT_PRESS_TICKS {
            short_press_gap_ticks += 1;
        } else if pending_short_press {
            pending_short_press = false;
            short_press_gap_ticks = 0;
            selected_control = selected_control.next();
            status_dirty = true;
        }

        if title_dirty {
            write_title_line(&mut lcd, &mut delay, &channels[active_channel]);
            title_dirty = false;
        }

        if status_dirty {
            write_status_line(
                &mut lcd,
                &mut delay,
                &channels[active_channel],
                selected_control,
            );
            status_dirty = false;
        }

        delay.delay_ms(1u8);
    }
}
