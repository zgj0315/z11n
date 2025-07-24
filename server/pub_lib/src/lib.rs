#[derive(
    Debug, PartialEq, strum_macros::EnumString, strum_macros::Display, strum_macros::EnumIter,
)]
pub enum AgentState {
    Online,  // 在线
    Offline, // 离线
}

pub const DB_PATH: &str = "../db/z11n.sqlite";
pub const SOCKET_PATH: &str = "../db/socket";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        println!("{}", AgentState::Online);
    }
}
