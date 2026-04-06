use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ColumnFamily {
    Default,
    Metadata,
    Data,
    Index,
    Cache,
    History,
    Snapshot,
    Custom(String),
}

impl ColumnFamily {
    pub fn name(&self) -> &str {
        match self {
            ColumnFamily::Default => "default",
            ColumnFamily::Metadata => "metadata",
            ColumnFamily::Data => "data",
            ColumnFamily::Index => "index",
            ColumnFamily::Cache => "cache",
            ColumnFamily::History => "history",
            ColumnFamily::Snapshot => "snapshot",
            ColumnFamily::Custom(name) => name,
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "default" => ColumnFamily::Default,
            "metadata" => ColumnFamily::Metadata,
            "data" => ColumnFamily::Data,
            "index" => ColumnFamily::Index,
            "cache" => ColumnFamily::Cache,
            "history" => ColumnFamily::History,
            "snapshot" => ColumnFamily::Snapshot,
            custom => ColumnFamily::Custom(custom.to_string()),
        }
    }

    pub fn all_default() -> Vec<Self> {
        vec![
            ColumnFamily::Default,
            ColumnFamily::Metadata,
            ColumnFamily::Data,
            ColumnFamily::Index,
            ColumnFamily::Cache,
            ColumnFamily::History,
            ColumnFamily::Snapshot,
        ]
    }
}

impl AsRef<str> for ColumnFamily {
    fn as_ref(&self) -> &str {
        self.name()
    }
}

impl std::fmt::Display for ColumnFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
