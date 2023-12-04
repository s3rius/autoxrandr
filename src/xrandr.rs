#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct XrandrCmd {
    args: Vec<String>,
}

impl XrandrCmd {
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

    pub fn to_string(&self) -> String {
        format!("xrandr {}", self.args.join(" "))
    }
}
