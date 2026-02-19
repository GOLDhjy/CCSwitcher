use std::fmt::{Display, Formatter};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
#[value(rename_all = "lower")]
pub enum ProviderKind {
    Glm,
    #[default]
    Custom,
}

impl Display for ProviderKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderKind::Glm => write!(f, "glm"),
            ProviderKind::Custom => write!(f, "custom"),
        }
    }
}
