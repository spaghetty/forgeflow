use tokio::runtime::Runtime;

pub struct Agent {}

impl Agent {
    pub fn new() -> Result<Self> {
        let rt = Runtime::new()?;
        Ok(Agent {});
    }

    pub fn run(&self) -> Result<_> {}
}
