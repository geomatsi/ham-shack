/*
   Copyright 2018 Ilya Epifanov
   Copyright 2026 geomatsi@gmail.com

   Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
   http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
   http://opensource.org/licenses/MIT>, at your option. This file may not be
   copied, modified, or distributed except according to those terms.

   Portions of this code were rewritten and updated using OpenAI
   Codex GPT-5.4-High (a large language model by OpenAI).
*/

use bitflags::bitflags;
use embedded_hal_1::i2c::I2c;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    CommunicationError,
    InvalidParameter,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CrystalLoad {
    _6,
    _8,
    _10,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PLL {
    A,
    B,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FeedbackMultisynth {
    MSNA,
    MSNB,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Multisynth {
    MS0,
    MS1,
    MS2,
    MS3,
    MS4,
    MS5,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClockOutput {
    Clk0 = 0,
    Clk1,
    Clk2,
    Clk3,
    Clk4,
    Clk5,
    Clk6,
    Clk7,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OutputDivider {
    Div1 = 0,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}

const ADDRESS: u8 = 0b0110_0000;

impl PLL {
    pub fn multisynth(&self) -> FeedbackMultisynth {
        match *self {
            PLL::A => FeedbackMultisynth::MSNA,
            PLL::B => FeedbackMultisynth::MSNB,
        }
    }
}

trait FractionalMultisynth {
    fn base_addr(&self) -> u8;
    fn ix(&self) -> u8;
}

impl FractionalMultisynth for FeedbackMultisynth {
    fn base_addr(&self) -> u8 {
        match *self {
            FeedbackMultisynth::MSNA => 26,
            FeedbackMultisynth::MSNB => 34,
        }
    }

    fn ix(&self) -> u8 {
        match *self {
            FeedbackMultisynth::MSNA => 6,
            FeedbackMultisynth::MSNB => 7,
        }
    }
}

impl FractionalMultisynth for Multisynth {
    fn base_addr(&self) -> u8 {
        match *self {
            Multisynth::MS0 => 42,
            Multisynth::MS1 => 50,
            Multisynth::MS2 => 58,
            Multisynth::MS3 => 66,
            Multisynth::MS4 => 74,
            Multisynth::MS5 => 82,
        }
    }

    fn ix(&self) -> u8 {
        match *self {
            Multisynth::MS0 => 0,
            Multisynth::MS1 => 1,
            Multisynth::MS2 => 2,
            Multisynth::MS3 => 3,
            Multisynth::MS4 => 4,
            Multisynth::MS5 => 5,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Register {
    DeviceStatus = 0,
    OutputEnable = 3,
    Clk0 = 16,
    Clk1 = 17,
    Clk2 = 18,
    Clk3 = 19,
    Clk4 = 20,
    Clk5 = 21,
    Clk6 = 22,
    Clk7 = 23,
    PLLReset = 177,
    CrystalLoad = 183,
}

impl Register {
    fn addr(self) -> u8 {
        self as u8
    }
}

bitflags! {
    pub struct DeviceStatusBits: u8 {
        const SYS_INIT = 0b1000_0000;
        const LOL_B = 0b0100_0000;
        const LOL_A = 0b0010_0000;
        const LOS = 0b0001_0000;
    }
}

bitflags! {
    struct CrystalLoadBits: u8 {
        const RESERVED = 0b00_010010;
        const CL_6 = 0b01_000000;
        const CL_8 = 0b10_000000;
        const CL_10 = 0b11_000000;
    }
}

bitflags! {
    struct ClockControlBits: u8 {
        const CLK_PDN = 0b1000_0000;
        const MS_INT = 0b0100_0000;
        const MS_SRC = 0b0010_0000;
        const CLK_SRC_MS = 0b0000_1100;
        const CLK_DRV_8 = 0b0000_0011;
    }
}

bitflags! {
    struct PLLResetBits: u8 {
        const PLLB_RST = 0b1000_0000;
        const PLLA_RST = 0b0010_0000;
    }
}

impl ClockOutput {
    fn register(self) -> Register {
        match self {
            ClockOutput::Clk0 => Register::Clk0,
            ClockOutput::Clk1 => Register::Clk1,
            ClockOutput::Clk2 => Register::Clk2,
            ClockOutput::Clk3 => Register::Clk3,
            ClockOutput::Clk4 => Register::Clk4,
            ClockOutput::Clk5 => Register::Clk5,
            ClockOutput::Clk6 => Register::Clk6,
            ClockOutput::Clk7 => Register::Clk7,
        }
    }

    fn ix(self) -> u8 {
        self as u8
    }
}

impl OutputDivider {
    fn bits(self) -> u8 {
        self as u8
    }

    fn min_divider(desired_divider: u16) -> Result<Self, Error> {
        match 16 - (desired_divider.max(1) - 1).leading_zeros() {
            0 => Ok(Self::Div1),
            1 => Ok(Self::Div2),
            2 => Ok(Self::Div4),
            3 => Ok(Self::Div8),
            4 => Ok(Self::Div16),
            5 => Ok(Self::Div32),
            6 => Ok(Self::Div64),
            7 => Ok(Self::Div128),
            _ => Err(Error::InvalidParameter),
        }
    }

    pub fn divisor(self) -> u8 {
        match self {
            Self::Div1 => 1,
            Self::Div2 => 2,
            Self::Div4 => 4,
            Self::Div8 => 8,
            Self::Div16 => 16,
            Self::Div32 => 32,
            Self::Div64 => 64,
            Self::Div128 => 128,
        }
    }
}

fn i2c_error<E>(_: E) -> Error {
    Error::CommunicationError
}

pub struct Si5351Device<I2C> {
    i2c: I2C,
    address: u8,
    xtal_freq: u32,
    clk_enabled_mask: u8,
    ms_int_mode_mask: u8,
    ms_src_mask: u8,
}

pub trait Si5351 {
    fn init_adafruit_module(&mut self) -> Result<(), Error>;
    fn init(&mut self, xtal_load: CrystalLoad) -> Result<(), Error>;
    fn read_device_status(&mut self) -> Result<DeviceStatusBits, Error>;

    fn find_int_dividers_for_max_pll_freq(
        &self,
        max_pll_freq: u32,
        freq: u32,
    ) -> Result<(u16, OutputDivider), Error>;
    fn find_pll_coeffs_for_dividers(
        &self,
        total_div: u32,
        denom: u32,
        freq: u32,
    ) -> Result<(u8, u32), Error>;

    fn set_frequency(&mut self, pll: PLL, clk: ClockOutput, freq: u32) -> Result<(), Error>;
    fn set_clock_enabled(&mut self, clk: ClockOutput, enabled: bool);

    fn flush_output_enabled(&mut self) -> Result<(), Error>;
    fn flush_clock_control(&mut self, clk: ClockOutput) -> Result<(), Error>;

    fn setup_pll_int(&mut self, pll: PLL, mult: u8) -> Result<(), Error>;
    fn setup_pll(&mut self, pll: PLL, mult: u8, num: u32, denom: u32) -> Result<(), Error>;
    fn setup_multisynth_int(
        &mut self,
        ms: Multisynth,
        mult: u16,
        r_div: OutputDivider,
    ) -> Result<(), Error>;
    fn setup_multisynth(
        &mut self,
        ms: Multisynth,
        div: u16,
        num: u32,
        denom: u32,
        r_div: OutputDivider,
    ) -> Result<(), Error>;
    fn select_clock_pll(&mut self, clock: ClockOutput, pll: PLL);
}

impl<I2C> Si5351Device<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C, address_bit: bool, xtal_freq: u32) -> Self {
        Self {
            i2c,
            address: ADDRESS | u8::from(address_bit),
            xtal_freq,
            clk_enabled_mask: 0,
            ms_int_mode_mask: 0,
            ms_src_mask: 0,
        }
    }

    pub fn new_adafruit_module(i2c: I2C) -> Self {
        Self::new(i2c, false, 25_000_000)
    }

    fn write_ms_config<MS: FractionalMultisynth + Copy>(
        &mut self,
        ms: MS,
        int: u16,
        frac_num: u32,
        frac_denom: u32,
        r_div: OutputDivider,
    ) -> Result<(), Error> {
        if frac_denom == 0 || frac_num > 0xFFFFF || frac_denom > 0xFFFFF {
            return Err(Error::InvalidParameter);
        }

        let (p1, p2, p3) = if frac_num == 0 {
            (128 * int as u32 - 512, 0, 1)
        } else {
            let ratio = (128u64 * frac_num as u64 / frac_denom as u64) as u32;
            (
                128 * int as u32 + ratio - 512,
                128 * frac_num - frac_denom * ratio,
                frac_denom,
            )
        };

        self.write_synth_registers(
            ms,
            [
                ((p3 & 0x0000_FF00) >> 8) as u8,
                p3 as u8,
                ((p1 & 0x0003_0000) >> 16) as u8 | r_div.bits(),
                ((p1 & 0x0000_FF00) >> 8) as u8,
                p1 as u8,
                (((p3 & 0x000F_0000) >> 12) | ((p2 & 0x000F_0000) >> 16)) as u8,
                ((p2 & 0x0000_FF00) >> 8) as u8,
                p2 as u8,
            ],
        )?;

        if frac_num == 0 {
            self.ms_int_mode_mask |= 1u8 << ms.ix();
        } else {
            self.ms_int_mode_mask &= !(1u8 << ms.ix());
        }

        Ok(())
    }

    fn reset_pll(&mut self, pll: PLL) -> Result<(), Error> {
        self.write_register(
            Register::PLLReset,
            match pll {
                PLL::A => PLLResetBits::PLLA_RST.bits(),
                PLL::B => PLLResetBits::PLLB_RST.bits(),
            },
        )
    }

    fn read_register(&mut self, reg: Register) -> Result<u8, Error> {
        let mut buffer = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg.addr()], &mut buffer)
            .map_err(i2c_error)?;
        Ok(buffer[0])
    }

    fn write_register(&mut self, reg: Register, byte: u8) -> Result<(), Error> {
        self.i2c
            .write(self.address, &[reg.addr(), byte])
            .map_err(i2c_error)
    }

    fn write_synth_registers<MS: FractionalMultisynth>(
        &mut self,
        ms: MS,
        params: [u8; 8],
    ) -> Result<(), Error> {
        self.i2c
            .write(
                self.address,
                &[
                    ms.base_addr(),
                    params[0],
                    params[1],
                    params[2],
                    params[3],
                    params[4],
                    params[5],
                    params[6],
                    params[7],
                ],
            )
            .map_err(i2c_error)
    }
}

impl<I2C> Si5351 for Si5351Device<I2C>
where
    I2C: I2c,
{
    fn init_adafruit_module(&mut self) -> Result<(), Error> {
        self.init(CrystalLoad::_10)
    }

    fn init(&mut self, xtal_load: CrystalLoad) -> Result<(), Error> {
        loop {
            let device_status = self.read_device_status()?;
            if !device_status.contains(DeviceStatusBits::SYS_INIT) {
                break;
            }
        }

        self.flush_output_enabled()?;

        const CLK_REGS: [Register; 8] = [
            Register::Clk0,
            Register::Clk1,
            Register::Clk2,
            Register::Clk3,
            Register::Clk4,
            Register::Clk5,
            Register::Clk6,
            Register::Clk7,
        ];

        for reg in CLK_REGS {
            self.write_register(reg, ClockControlBits::CLK_PDN.bits())?;
        }

        let crystal_load = CrystalLoadBits::RESERVED
            | match xtal_load {
                CrystalLoad::_6 => CrystalLoadBits::CL_6,
                CrystalLoad::_8 => CrystalLoadBits::CL_8,
                CrystalLoad::_10 => CrystalLoadBits::CL_10,
            };
        self.write_register(Register::CrystalLoad, crystal_load.bits())
    }

    fn read_device_status(&mut self) -> Result<DeviceStatusBits, Error> {
        Ok(DeviceStatusBits::from_bits_truncate(
            self.read_register(Register::DeviceStatus)?,
        ))
    }

    fn find_int_dividers_for_max_pll_freq(
        &self,
        max_pll_freq: u32,
        freq: u32,
    ) -> Result<(u16, OutputDivider), Error> {
        let total_divider = (max_pll_freq / freq) as u16;
        let r_div = OutputDivider::min_divider(total_divider / 900)?;
        let ms_div = (total_divider / (2 * r_div.divisor() as u16) * 2).max(6);
        if ms_div > 1800 {
            return Err(Error::InvalidParameter);
        }
        Ok((ms_div, r_div))
    }

    fn find_pll_coeffs_for_dividers(
        &self,
        total_div: u32,
        denom: u32,
        freq: u32,
    ) -> Result<(u8, u32), Error> {
        if denom == 0 || denom > 0xFFFFF {
            return Err(Error::InvalidParameter);
        }

        let pll_freq = freq * total_div;
        let mult = (pll_freq / self.xtal_freq) as u8;
        let frac =
            ((pll_freq % self.xtal_freq) as u64 * denom as u64 / self.xtal_freq as u64) as u32;

        Ok((mult, frac))
    }

    fn set_frequency(&mut self, pll: PLL, clk: ClockOutput, freq: u32) -> Result<(), Error> {
        let denom = 1_048_575;
        let (ms_divider, r_div) = self.find_int_dividers_for_max_pll_freq(900_000_000, freq)?;
        let total_div = ms_divider as u32 * r_div.divisor() as u32;
        let (mult, num) = self.find_pll_coeffs_for_dividers(total_div, denom, freq)?;

        let ms = match clk {
            ClockOutput::Clk0 => Multisynth::MS0,
            ClockOutput::Clk1 => Multisynth::MS1,
            ClockOutput::Clk2 => Multisynth::MS2,
            ClockOutput::Clk3 => Multisynth::MS3,
            ClockOutput::Clk4 => Multisynth::MS4,
            ClockOutput::Clk5 => Multisynth::MS5,
            ClockOutput::Clk6 | ClockOutput::Clk7 => return Err(Error::InvalidParameter),
        };

        self.setup_pll(pll, mult, num, denom)?;
        self.setup_multisynth_int(ms, ms_divider, r_div)?;
        self.select_clock_pll(clk, pll);
        self.set_clock_enabled(clk, true);
        self.flush_clock_control(clk)?;
        self.reset_pll(pll)?;
        self.flush_output_enabled()
    }

    fn set_clock_enabled(&mut self, clk: ClockOutput, enabled: bool) {
        let bit = 1u8 << clk.ix();
        if enabled {
            self.clk_enabled_mask |= bit;
        } else {
            self.clk_enabled_mask &= !bit;
        }
    }

    fn flush_output_enabled(&mut self) -> Result<(), Error> {
        self.write_register(Register::OutputEnable, !self.clk_enabled_mask)
    }

    fn flush_clock_control(&mut self, clk: ClockOutput) -> Result<(), Error> {
        let bit = 1u8 << clk.ix();
        let clk_control_pdn = if self.clk_enabled_mask & bit != 0 {
            ClockControlBits::empty()
        } else {
            ClockControlBits::CLK_PDN
        };
        let ms_int_mode = if self.ms_int_mode_mask & bit == 0 {
            ClockControlBits::empty()
        } else {
            ClockControlBits::MS_INT
        };
        let ms_src = if self.ms_src_mask & bit == 0 {
            ClockControlBits::empty()
        } else {
            ClockControlBits::MS_SRC
        };
        let base = ClockControlBits::CLK_SRC_MS | ClockControlBits::CLK_DRV_8;

        self.write_register(
            clk.register(),
            (clk_control_pdn | ms_int_mode | ms_src | base).bits(),
        )
    }

    fn setup_pll_int(&mut self, pll: PLL, mult: u8) -> Result<(), Error> {
        self.setup_pll(pll, mult, 0, 1)
    }

    fn setup_pll(&mut self, pll: PLL, mult: u8, num: u32, denom: u32) -> Result<(), Error> {
        if !(15..=90).contains(&mult) {
            return Err(Error::InvalidParameter);
        }

        self.write_ms_config(
            pll.multisynth(),
            mult.into(),
            num,
            denom,
            OutputDivider::Div1,
        )
    }

    fn setup_multisynth_int(
        &mut self,
        ms: Multisynth,
        mult: u16,
        r_div: OutputDivider,
    ) -> Result<(), Error> {
        self.setup_multisynth(ms, mult, 0, 1, r_div)
    }

    fn setup_multisynth(
        &mut self,
        ms: Multisynth,
        div: u16,
        num: u32,
        denom: u32,
        r_div: OutputDivider,
    ) -> Result<(), Error> {
        if !(6..=1800).contains(&div) {
            return Err(Error::InvalidParameter);
        }

        self.write_ms_config(ms, div, num, denom, r_div)
    }

    fn select_clock_pll(&mut self, clock: ClockOutput, pll: PLL) {
        let bit = 1u8 << clock.ix();
        match pll {
            PLL::A => self.ms_src_mask &= !bit,
            PLL::B => self.ms_src_mask |= bit,
        }
    }
}
