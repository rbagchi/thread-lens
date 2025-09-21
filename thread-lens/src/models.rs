use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum FrameCategory {
    Jvm,
    Framework,
    Application,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategorizedFrame {
    pub line: String,
    pub category: FrameCategory,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ThreadCategory {
    Jvm,
    Framework,
    Application,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NormalizedThread {
    pub name: String,
    pub state: String, // Consider making this an enum later
    pub category: ThreadCategory,
    pub frames: Vec<CategorizedFrame>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadDump {
    pub jvm_version: String,
    pub timestamp: DateTime<Utc>,
    pub threads: Vec<NormalizedThread>,
}