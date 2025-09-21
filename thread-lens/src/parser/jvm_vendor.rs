use serde::Serialize;

#[derive(Serialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum JvmVendor {
    OpenJDK,
    IBM,
    Unknown,
}
