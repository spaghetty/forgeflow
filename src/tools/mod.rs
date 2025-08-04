// 1. Declare your actuator files as public sub-modules
pub mod gmail_actions;
pub mod simple_file_writer;

// 2. Publicly re-export the structs so users can access them easily
pub use simple_file_writer::SimpleFileWriter;
