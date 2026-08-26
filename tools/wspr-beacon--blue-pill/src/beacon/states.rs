#[derive(Clone, Copy, PartialEq)]
pub enum State {
    GpsWait,
    TxWait,
    TxCalib,
    TxActive,
    Error(ErrorState),
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
            ErrorState::PPSQueueFailure => "PPS IRQ queue failure",
        }
    }
}

pub enum WSPRError {
    Si5351Error(si5351::Error),
    CalibUnexpectedState,
    TxQueueError,
    TxPPSError,
}

impl From<si5351::Error> for WSPRError {
    fn from(e: si5351::Error) -> Self {
        WSPRError::Si5351Error(e)
    }
}
