use x11rb::{
    protocol::{
        randr::{ConnectionExt, Rotation},
        xproto::Screen,
    },
    rust_connection::RustConnection,
};

use crate::xrandr::XrandrCmd;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Output {
    pub name: String,
    pub is_primary: bool,
    pub is_connected: bool,
    pub mapped: bool,
    pub position: Option<(i16, i16)>,
    pub rotate: u16,
    pub mode: Option<(u16, u16)>,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct State {
    pub outputs: Vec<Output>,
}

impl State {
    pub fn from_screen(conn: &RustConnection, screen: &Screen) -> anyhow::Result<Self> {
        let get_screen = conn.randr_get_screen_resources(screen.root)?.reply()?;
        let primary = conn.randr_get_output_primary(screen.root)?.reply()?.output;
        let mut outputs = Vec::new();
        for output in &get_screen.outputs {
            let output_info = conn
                .randr_get_output_info(*output, x11rb::CURRENT_TIME)?
                .reply()?;
            let is_connected =
                output_info.connection == x11rb::protocol::randr::Connection::CONNECTED;
            let name = String::from_utf8_lossy(output_info.name.as_slice()).to_string();

            let mut position = None;
            let mut mode = None;
            let mut rotate = Rotation::ROTATE0;
            let mut mapped = false;

            if output_info.crtc != 0 {
                let crtc = conn
                    .randr_get_crtc_info(output_info.crtc, x11rb::CURRENT_TIME)?
                    .reply()?;
                position = Some((crtc.x, crtc.y));
                mode = Some((crtc.width, crtc.height));
                rotate = crtc.rotation;
                mapped = true;
            }

            outputs.push(Output {
                name,
                is_primary: *output == primary,
                is_connected,
                position,
                mode,
                mapped,
                rotate: rotate.into(),
            });
        }
        outputs.sort_by(|left, right| right.name.cmp(&left.name));
        Ok(State { outputs })
    }

    #[must_use]
    pub fn should_map(&self) -> bool {
        for output in &self.outputs {
            if output.is_connected && !output.mapped {
                return true;
            }
        }
        false
    }

    #[must_use]
    pub fn should_unmap(&self) -> bool {
        for output in &self.outputs {
            if !output.is_connected && output.mapped {
                return true;
            }
        }
        false
    }
    #[must_use]
    pub fn outputs_sign(&self) -> String {
        self.outputs
            .iter()
            .filter(|out| out.is_connected)
            .map(|out| out.name.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }

    #[must_use]
    pub fn to_xrandr_cmd(&self) -> XrandrCmd {
        let mut args = Vec::new();
        for output in &self.outputs {
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
                // args.push(format!("--mode {}x{}", width, height));
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
        XrandrCmd::new(args)
    }
}
