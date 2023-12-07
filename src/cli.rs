#[derive(Clone, Debug, clap::Parser)]
pub struct Cli {
    #[arg(short, long, default_value = "false")]
    pub background: bool,

    /// How often to check for changes in seconds.
    #[arg(long, default_value = "1")]
    pub delay: u64,

    /// Override screen to connect to.
    /// By default the DISPLAY environment variable is used.
    #[arg(long)]
    pub display: Option<String>,

    /// Command to execute when outputs are remapped.
    #[arg(long)]
    pub on_remap: Option<String>,

    /// Reapply configuration first, then start listening for changes.
    #[arg(long)]
    pub reapply: bool,
}
