pub mod app;
pub mod db;
pub mod domains;
pub mod state;
pub mod utils;

// Re-export commonly used types
pub use state::SharedAppState;
