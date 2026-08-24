use core::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    GpsWait,
    TxWait,
    TxCalib,
    TxActive,
    Error(ErrorState),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::GpsWait => write!(f, "GPS Wait"),
            State::TxWait => write!(f, "TX Wait"),
            State::TxCalib => write!(f, "Calibration"),
            State::TxActive => write!(f, "Transmitting"),
            State::Error(code) => write!(f, "Err:{}", code), // Dynamically formats the number!
        }
    }
}

impl State {
    pub fn as_str(&self) -> &'static str {
        match self {
            State::GpsWait => "GPS Wait",
            State::TxWait => "TX Wait",
            State::TxCalib => "Calibration",
            State::TxActive => "TX",
            State::Error(_) => "Error",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ErrorState {
    WSPRTxFailure,
    PPSQueueFailure,
    CalibFailure,
}

impl ErrorState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorState::CalibFailure => "Calibration failure",
            ErrorState::WSPRTxFailure => "WSPR Tx failure",
            ErrorState::PPSQueueFailure => "PPS queue",
        }
    }
}

impl fmt::Display for ErrorState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            ErrorState::CalibFailure => "Calibration failure",
            ErrorState::WSPRTxFailure => "WSPR Tx failure",
            ErrorState::PPSQueueFailure => "PPS IRQ queue failure",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub enum WSPRError {
    Si5351Error(si5351::Error),
    CalibUnexpectedState,
    TxQueueError,
}

impl From<si5351::Error> for WSPRError {
    fn from(e: si5351::Error) -> Self {
        WSPRError::Si5351Error(e)
    }
}
