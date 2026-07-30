use crate::beacon::states::State;

#[derive(Clone, Copy, PartialEq)]
pub struct Status {
    pub state: State,
    pub qth: Option<[u8; 4]>,
}

impl Status {
    pub fn qth_str(&self) -> Option<&str> {
        self.qth
            .as_ref()
            .and_then(|qth| core::str::from_utf8(qth).ok())
    }
}
