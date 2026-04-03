/// Shared mutable runtime state passed through the module pipeline.
#[derive(Debug, Default)]
pub struct State {
    pub total:   u32,
    pub success: u32,
    pub failed:  u32,
    pub reboot_required: bool,
}