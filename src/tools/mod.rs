// The `tools` module provides a collection of tools that can be used by the agent.

// 1. Declare your actuator files as public sub-modules
pub mod daily_summary_writer;
pub mod gmail_actions;
pub mod simple_file_writer;

// 2. Publicly re-export the structs so users can access them easily
pub use daily_summary_writer::DailySummaryWriter;
pub use simple_file_writer::SimpleFileWriter;
