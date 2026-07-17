use uuid::Uuid;

pub const DEFAULT_IMAGE: &str = "ghcr.io/ezygang/vulcanum/agent:latest";
pub const MACOS_DOCKER_DESKTOP_CLI_PATH: &str =
    "/Applications/Docker.app/Contents/Resources/bin/docker";
pub const MAX_WORKER_CAPACITY: i32 = 3;
pub const DEFAULT_TEAM_ID: Uuid = Uuid::from_u128(1);
