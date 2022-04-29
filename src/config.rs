use std::fmt;

use redis::{IntoConnectionInfo, RedisError};

use crate::{CreatePoolError, Pool, PoolBuilder, PoolConfig, Runtime};

#[derive(Clone, Debug)]
pub struct Config<T: IntoConnectionInfo + Clone> {
    pub nodes: Vec<T>,
    pub pool: Option<PoolConfig>,
}

impl<T: IntoConnectionInfo + Clone> Config<T> {
    pub fn create_pool(&self, runtime: Option<Runtime>) -> Result<Pool, CreatePoolError> {
        let mut builder = self.builder().map_err(CreatePoolError::Config)?;
        if let Some(runtime) = runtime {
            builder = builder.runtime(runtime);
        }
        builder.build().map_err(CreatePoolError::Build)
    }

    pub fn builder(&self) -> Result<PoolBuilder, ConfigError> {
        let manager = crate::Manager::new(self.nodes.clone())?;
        let pool_config = self.get_pool_config();
        Ok(Pool::builder(manager).config(pool_config))
    }

    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.unwrap_or_default()
    }

    #[must_use]
    pub fn from_nodes(nodes: Vec<T>) -> Config<T> {
        Config {
            nodes: nodes.into(),
            pool: None,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Redis(RedisError),
}

impl From<RedisError> for ConfigError {
    fn from(e: RedisError) -> Self {
        Self::Redis(e)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Redis(e) => write!(f, "Redis: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}
