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
    CALIBQueueFailure,
    WSPRQueueFailure,
    PPSQueueFailure,
}

impl ErrorState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorState::CALIBQueueFailure => "CALIB queue",
            ErrorState::WSPRQueueFailure => "WSPR queue",
            ErrorState::PPSQueueFailure => "PPS queue",
        }
    }
}

impl fmt::Display for ErrorState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            ErrorState::CALIBQueueFailure => "CALIB queue failure",
            ErrorState::WSPRQueueFailure => "WSPR queue failure",
            ErrorState::PPSQueueFailure => "PPS IRQ queue failure",
        };
        write!(f, "{}", s)
    }
}
