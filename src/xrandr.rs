#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct XrandrCmd {
    args: Vec<String>,
}

impl XrandrCmd {
    #[must_use]
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    pub fn exec(&self) -> anyhow::Result<()> {
        let mut cmd = std::process::Command::new("xrandr");
        for arg in &self.args {
            cmd.arg(arg.clone());
        }
        cmd.spawn()?.wait()?;
        Ok(())
    }
}

impl ToString for XrandrCmd {
    fn to_string(&self) -> String {
        format!("xrandr {}", self.args.join(" "))
    }
}
