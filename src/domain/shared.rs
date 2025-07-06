//! 共有カーネル
//! 
//! 複数の集約で使用される共通の値オブジェクトとイベントを定義

pub mod value_objects;
pub mod events;

pub use value_objects::Entity; 