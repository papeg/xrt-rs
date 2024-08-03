use crate::HardwareDatatype;

pub enum ArgumentType {
    Direct(Box<dyn HardwareDatatype>),
    Buffered(Box<dyn HardwareDatatype>),
}
