#[derive(
    Debug, PartialEq, strum_macros::EnumString, strum_macros::Display, strum_macros::EnumIter,
)]
pub enum AgentState {
    Online,  // 在线
    Offline, // 离线
}

pub const DATA_DIR: &str = "./data";
pub const DB_DIR: &str = "../db";
pub const DB_PATH: &str = "../db/z11n.sqlite";
pub const UDS_PATH: &str = "../db/uds";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        println!("{}", AgentState::Online);
    }
}
