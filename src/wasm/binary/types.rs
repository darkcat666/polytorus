#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    I32,
    I64,
}

impl From<u8> for ValueType {
    fn from(value: u8) -> Self {
        match value {
            0x7F => Self::I32,
            0x7E => Self::I64,
            _ => panic!("invalid value type: {:X}", value),
        }
    }
}
