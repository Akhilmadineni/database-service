use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use crate::error::ConfigError;

#[derive(Clone, Debug)]
pub struct ApiSettings {
    pub service_name: String,
    pub bind_addr: IpAddr,
    pub port: u16,
}

impl ApiSettings {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            service_name: env::var("API_SERVICE_NAME")
                .unwrap_or_else(|_| String::from("database-service-api")),
            bind_addr: read_ip_addr("API_BIND_ADDR")?.unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            port: read_u16("API_PORT")?.unwrap_or(8080),
        })
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind_addr, self.port)
    }
}

#[derive(Clone, Debug)]
pub struct WorkerSettings {
    pub service_name: String,
    pub poll_interval_secs: u64,
}

impl WorkerSettings {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            service_name: env::var("WORKER_SERVICE_NAME")
                .unwrap_or_else(|_| String::from("database-service-worker")),
            poll_interval_secs: read_u64("WORKER_POLL_INTERVAL_SECS")?.unwrap_or(30),
        })
    }
}

fn read_ip_addr(key: &'static str) -> Result<Option<IpAddr>, ConfigError> {
    match env::var(key) {
        Ok(value) => value
            .parse::<IpAddr>()
            .map(Some)
            .map_err(|error| ConfigError::InvalidEnv {
                key,
                message: error.to_string(),
            }),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(ConfigError::InvalidEnv {
            key,
            message: error.to_string(),
        }),
    }
}

fn read_u16(key: &'static str) -> Result<Option<u16>, ConfigError> {
    match env::var(key) {
        Ok(value) => value
            .parse::<u16>()
            .map(Some)
            .map_err(|error| ConfigError::InvalidEnv {
                key,
                message: error.to_string(),
            }),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(ConfigError::InvalidEnv {
            key,
            message: error.to_string(),
        }),
    }
}

fn read_u64(key: &'static str) -> Result<Option<u64>, ConfigError> {
    match env::var(key) {
        Ok(value) => value
            .parse::<u64>()
            .map(Some)
            .map_err(|error| ConfigError::InvalidEnv {
                key,
                message: error.to_string(),
            }),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(ConfigError::InvalidEnv {
            key,
            message: error.to_string(),
        }),
    }
}
