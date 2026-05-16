use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingJob {
    pub id: String,
    pub status: JobStatus,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Extracting,
    Enhancing,
    Normalizing,
    Remuxing,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementPreset {
    pub name: String,
    pub target_lufs: f32,
    pub true_peak_max_dbfs: f32,
    pub denoise_intensity: f32,
}
