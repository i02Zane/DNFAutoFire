//! 核心按键发送模块

pub mod autofire;
pub mod autorun;
pub mod classes;
pub mod combo;
pub mod detection;
pub mod hook;
pub mod keyboard;
pub mod window;

pub use autofire::{AutoFireEngine, FireKeyConfig};
pub use autorun::AutoRunEngine;
pub use combo::ComboEngine;
pub use detection::DetectionRuntime;
#[allow(unused_imports)]
pub use keyboard::vk;
