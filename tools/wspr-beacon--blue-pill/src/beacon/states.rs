use core::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    GpsWait,
    TxWait,
    TxReady,
    TxActive,
    Error(ErrorState),
}

#[derive(Clone, Copy, PartialEq)]
pub enum ErrorState {
    WSPRQueueFailure,
    PPSQueueFailure,
}

impl fmt::Display for ErrorState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            ErrorState::WSPRQueueFailure => "WSPR queue failure",
            ErrorState::PPSQueueFailure => "PPS IRQ queue failure",
        };
        write!(f, "{}", s)
    }
}
