use bitbang_hal::i2c;
use embedded_hal::blocking::i2c::{
    Read as Eh02Read, Write as Eh02Write, WriteRead as Eh02WriteRead,
};
use embedded_hal_1::i2c::{
    Error as Eh1Error, ErrorKind, ErrorType, I2c, NoAcknowledgeSource, Operation,
};

#[derive(Debug)]
pub enum BitBangI2cError<E> {
    Bus(E),
    NoAcknowledge,
    InvalidData,
    UnsupportedTransaction,
}

impl<E> From<i2c::Error<E>> for BitBangI2cError<E> {
    fn from(error: i2c::Error<E>) -> Self {
        match error {
            i2c::Error::Bus(inner) => Self::Bus(inner),
            i2c::Error::NoAck => Self::NoAcknowledge,
            i2c::Error::InvalidData => Self::InvalidData,
        }
    }
}

impl<E: core::fmt::Debug> Eh1Error for BitBangI2cError<E> {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Bus(_) => ErrorKind::Bus,
            Self::NoAcknowledge => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Unknown),
            Self::InvalidData | Self::UnsupportedTransaction => ErrorKind::Other,
        }
    }
}

pub struct Eh1BitBangI2c<T> {
    inner: T,
}

impl<T> Eh1BitBangI2c<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T, E> ErrorType for Eh1BitBangI2c<T>
where
    T: Eh02Read<Error = i2c::Error<E>>
        + Eh02Write<Error = i2c::Error<E>>
        + Eh02WriteRead<Error = i2c::Error<E>>,
    E: core::fmt::Debug,
{
    type Error = BitBangI2cError<E>;
}

impl<T, E> I2c for Eh1BitBangI2c<T>
where
    T: Eh02Read<Error = i2c::Error<E>>
        + Eh02Write<Error = i2c::Error<E>>
        + Eh02WriteRead<Error = i2c::Error<E>>,
    E: core::fmt::Debug,
{
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        match operations {
            [] => Ok(()),
            [Operation::Read(read)] => self.inner.read(address, read).map_err(Into::into),
            [Operation::Write(write)] => self.inner.write(address, write).map_err(Into::into),
            [Operation::Write(write), Operation::Read(read)] => self
                .inner
                .write_read(address, write, read)
                .map_err(Into::into),
            _ => Err(BitBangI2cError::UnsupportedTransaction),
        }
    }
}
