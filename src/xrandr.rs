use crate::state::State;

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

impl From<&State> for XrandrCmd {
    fn from(value: &State) -> Self {
        let mut args = Vec::new();
        for output in &value.outputs {
            args.extend([String::from("--output"), output.name.clone()]);
            if !output.is_connected {
                args.push("--off".into());
                continue;
            }
            if output.is_primary {
                args.push("--primary".into());
            }
            if let Some((width, height)) = output.mode {
                args.extend([String::from("--mode"), format!("{width}x{height}")]);
            }
            if let Some((x, y)) = output.position {
                args.extend([String::from("--pos"), format!("{x}x{y}")]);
            }
            let rotate = match output.rotate {
                2 => "left",
                4 => "inverted",
                8 => "right",
                _ => "normal",
            };
            args.extend([String::from("--rotate"), rotate.into()]);
        }
        Self::new(args)
    }
}

impl ToString for XrandrCmd {
    fn to_string(&self) -> String {
        format!("xrandr {}", self.args.join(" "))
    }
}
