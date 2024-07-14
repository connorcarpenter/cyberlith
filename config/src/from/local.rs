#[allow(dead_code)]
pub const SELF_BINDING_ADDR: &str = "127.0.0.1";

#[allow(dead_code)]
pub const PUBLIC_PROTOCOL: &str = "http";

#[allow(dead_code)]
pub const PUBLIC_IP_ADDR: &str = "127.0.0.1";

#[allow(dead_code)]
pub const CONTENT_SERVER_RECV_ADDR: &str = "127.0.0.1";

#[allow(dead_code)]
pub const REGION_SERVER_RECV_ADDR: &str = "127.0.0.1";

#[allow(dead_code)]
pub const SESSION_SERVER_RECV_ADDR: &str = "127.0.0.1";

#[allow(dead_code)]
pub const WORLD_SERVER_RECV_ADDR: &str = "127.0.0.1";

#[allow(dead_code)]
pub const ASSET_SERVER_RECV_ADDR: &str = "127.0.0.1";

#[allow(dead_code)]
pub const SOCIAL_SERVER_RECV_ADDR: &str = "127.0.0.1";

#[allow(dead_code)]
pub const AUTH_SERVER_RECV_ADDR: &str = "127.0.0.1";

#[allow(dead_code)]
pub const CONTENT_SERVER_FILES_PATH: &str = "./files";

#[allow(dead_code)]
pub const ASSET_SERVER_FILES_PATH: &str = "./assets";

#[allow(dead_code)]
pub const REDIRECTOR_PORT: u16 = 14195;

#[allow(dead_code)]
pub const GATEWAY_PORT: u16 = 14196;

#[allow(dead_code)]
pub const CONTENT_SERVER_PORT: u16 = 14197;

#[allow(dead_code)]
pub const REGION_SERVER_PORT: u16 = 14198;

#[allow(dead_code)]
pub const SESSION_SERVER_HTTP_PORT: u16 = 14199;

#[allow(dead_code)]
pub const SESSION_SERVER_SIGNAL_PORT: u16 = 14200;

#[allow(dead_code)]
pub const SESSION_SERVER_WEBRTC_PORT: u16 = 14201;

#[allow(dead_code)]
pub const WORLD_SERVER_HTTP_PORT: u16 = 14202;

#[allow(dead_code)]
pub const WORLD_SERVER_SIGNAL_PORT: u16 = 14203;

#[allow(dead_code)]
pub const WORLD_SERVER_WEBRTC_PORT: u16 = 14204;

#[allow(dead_code)]
pub const ASSET_SERVER_PORT: u16 = 14205;

#[allow(dead_code)]
pub const AUTH_SERVER_PORT: u16 = 14206;

#[allow(dead_code)]
pub const SOCIAL_SERVER_PORT: u16 = 14207;

#[allow(dead_code)]
pub const REGION_SERVER_SECRET: &str = "ArQZmRSf4xvbLVusVjrqGhIaZOExAeIq";

#[allow(dead_code)]
pub const SESSION_SERVER_GLOBAL_SECRET: &str = "zUe6K0RKY03JJMPo3u5SaByfiut0alOW";

#[allow(dead_code)]
pub const WORLD_SERVER_GLOBAL_SECRET: &str = "VKHusVjrGh035aSlQ7236bvVxlQ70alOW";

#[allow(dead_code)]
pub const ASSET_SERVER_GLOBAL_SECRET: &str = "QvsVjrGh035V70aVKHuaSbxlllQ7236OW";

#[allow(dead_code)]
pub const SOCIAL_SERVER_GLOBAL_SECRET: &str = "sVjrSbaVKHuaSbxlGh03QvsVjrSbxl";

// cpu priorities
pub use crate::from::cpu_priority::{
    ASSET_SERVER_CPU_PRIORITY, AUTH_SERVER_CPU_PRIORITY, CONTENT_SERVER_CPU_PRIORITY,
    GATEWAY_SERVER_CPU_PRIORITY, REGION_SERVER_CPU_PRIORITY, SESSION_SERVER_CPU_PRIORITY,
    SOCIAL_SERVER_CPU_PRIORITY, WORLD_SERVER_CPU_PRIORITY,
};

#[allow(dead_code)]
pub const TOTAL_CPU_PRIORITY: usize = REGION_SERVER_CPU_PRIORITY
    + AUTH_SERVER_CPU_PRIORITY
    + CONTENT_SERVER_CPU_PRIORITY
    + GATEWAY_SERVER_CPU_PRIORITY
    + ASSET_SERVER_CPU_PRIORITY
    + SOCIAL_SERVER_CPU_PRIORITY
    + SESSION_SERVER_CPU_PRIORITY
    + WORLD_SERVER_CPU_PRIORITY;
