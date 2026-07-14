#[derive(Clone, Copy, PartialEq)]
pub enum State {
    GpsWait,
    TxWait,
    TxReady,
    TxActive,
    TxDone,
    Error(u8),
}
