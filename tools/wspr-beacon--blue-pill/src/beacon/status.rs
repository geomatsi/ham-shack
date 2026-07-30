use crate::beacon::states::State;

#[derive(Clone, Copy, PartialEq)]
pub struct Status {
    pub state: State,
    pub qth: Option<[u8; 4]>,
}
