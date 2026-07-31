//! 推理引擎模块

pub mod recommender;
pub mod types;

pub use recommender::{merge_recommendations, recommend};
