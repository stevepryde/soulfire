use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "message")]
pub enum CommandError {
    Locked,
    InvalidInput(String),
    Core(String),
}

impl From<soulfire_core::error::CoreError> for CommandError {
    fn from(value: soulfire_core::error::CoreError) -> Self {
        CommandError::Core(value.to_string())
    }
}
