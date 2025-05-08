use spectre_hashes::Hash as SpectreHash;
use serde::{Deserialize, Serialize};
use spectre_cli::cli_args::CliArgs;
use utoipa::ToSchema;

#[derive(ToSchema, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub cli_args: CliArgs,
    pub net_bps: u8,
    pub net_tps_max: u16,
    #[schema(value_type = String)]
    pub checkpoint: SpectreHash,
    pub disable_vcp_wait_for_sync: bool,
}
