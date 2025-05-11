pub trait TurboActionSerialization: Sized {
    fn deserialize(action: &[u8]) -> Result<(Self, &[u8]), &'static str>;
    fn serialize_json(json_str: &str) -> Result<Vec<u8>, &'static str>;
}

pub trait HasTerminalState {
    fn is_terminal(&self) -> bool;
}

pub trait HasActions {
    fn actions(&self) -> Vec<u8>;
}
