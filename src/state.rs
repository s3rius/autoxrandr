use byteorder::ByteOrder;
use x11rb::{
    protocol::{
        randr::{ConnectionExt as RandRext, Rotation},
        xproto::ConnectionExt as XprotoExt,
    },
    rust_connection::RustConnection,
};

use crate::xrandr::XrandrCmd;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Output {
    pub name: String,
    pub edid: u64,
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
    /// Get EDID code from output
    /// The EDID is a 128 byte array, that contains the following information:
    /// 0-7: header
    /// 8-9: manufacturer code
    /// 10-11: product code
    /// 12-15: serial number
    /// We skip the header and combine the manufacturer code, product code and serial number
    /// into a single u64. This is the display's unique code. We use this to identify displays.
    /// This should be unique enough for our purposes. Although it is possible that two displays
    /// have the same manufacturer code, product code and serial number, it's fine.
    fn get_display_edid(conn: &RustConnection, output: u32) -> anyhow::Result<u64> {
        for atom in conn.randr_list_output_properties(output)?.reply()?.atoms {
            let name = conn.get_atom_name(atom)?.reply()?.name;
            // We only care about the EDID property.
            if name == b"EDID" {
                let edid = conn
                    .randr_get_output_property::<u32>(output, atom, 19, 0, 128, false, false)?
                    .reply()?;
                let manufacturer_code = byteorder::LittleEndian::read_u16(&edid.data[8..10]);
                let product_code = byteorder::LittleEndian::read_u16(&edid.data[10..12]);
                let serial_number = byteorder::LittleEndian::read_u32(&edid.data[12..16]);
                // We shift first number 32 bits to the left, making room for the next 32 bits.
                let display_ucode = (serial_number as u64) << 32
                    // We shift the second number 16 bits to the left, making room for the next 16 bits.
                    | (product_code as u64) << 16
                    // We don't need to shift the last number, because it's the last 32 bits.
                    | (manufacturer_code as u64);
                return Ok(display_ucode);
            }
        }
        Ok(0)
    }

    pub fn current(
        conn: &RustConnection,
        root: u32,
        connected_outputs: &Vec<u32>,
    ) -> anyhow::Result<Self> {
        let primary = conn.randr_get_output_primary(root)?.reply()?.output;
        let mut outputs = Vec::new();
        for output in connected_outputs {
            let display_ucode = Self::get_display_edid(conn, *output)?;
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
                edid: display_ucode,
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
            .map(|out| format!("{}-{}", out.name, out.edid))
            .collect::<Vec<_>>()
            .join(",")
    }
}
