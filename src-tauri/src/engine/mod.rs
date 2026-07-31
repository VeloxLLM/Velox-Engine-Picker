//! 推理引擎模块

pub mod recommender;
pub mod types;

pub use recommender::recommend;
pub use types::{
    BackendType, EngineBackendPair, EngineRecommendation, InferenceEngine,
};