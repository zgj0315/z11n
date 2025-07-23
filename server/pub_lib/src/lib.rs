#[derive(
    Debug, PartialEq, strum_macros::EnumString, strum_macros::Display, strum_macros::EnumIter,
)]
pub enum AgentState {
    Online,  // 在线
    Offline, // 离线
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        println!("{}", AgentState::Online);
    }
}
