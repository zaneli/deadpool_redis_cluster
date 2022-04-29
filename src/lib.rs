mod config;

use std::ops::{Deref, DerefMut};

use deadpool::{async_trait, managed};
use redis::{
    cluster::ClusterClient, cluster::ClusterConnection, ConnectionLike, IntoConnectionInfo,
    RedisError, RedisResult, Value,
};

pub use redis;

pub use self::config::{Config, ConfigError};

pub use deadpool::managed::reexports::*;
deadpool::managed_reexports!(
    "redis_cluster",
    Manager,
    Connection,
    RedisError,
    ConfigError
);

type RecycleResult = managed::RecycleResult<RedisError>;

pub struct Connection {
    conn: Object,
}

impl Connection {
    #[must_use]
    pub fn take(this: Self) -> ClusterConnection {
        Object::take(this.conn)
    }
}

impl From<Object> for Connection {
    fn from(conn: Object) -> Self {
        Self { conn }
    }
}

impl Deref for Connection {
    type Target = ClusterConnection;

    fn deref(&self) -> &ClusterConnection {
        &self.conn
    }
}

impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut ClusterConnection {
        &mut self.conn
    }
}

impl AsRef<ClusterConnection> for Connection {
    fn as_ref(&self) -> &ClusterConnection {
        &self.conn
    }
}

impl AsMut<ClusterConnection> for Connection {
    fn as_mut(&mut self) -> &mut ClusterConnection {
        &mut self.conn
    }
}

impl ConnectionLike for Connection {
    fn req_packed_command(&mut self, cmd: &[u8]) -> RedisResult<Value> {
        self.conn.req_packed_command(cmd)
    }

    fn req_packed_commands(
        &mut self,
        cmd: &[u8],
        offset: usize,
        count: usize,
    ) -> RedisResult<Vec<Value>> {
        self.conn.req_packed_commands(cmd, offset, count)
    }

    fn get_db(&self) -> i64 {
        self.conn.get_db()
    }

    fn is_open(&self) -> bool {
        self.conn.is_open()
    }

    fn check_connection(&mut self) -> bool {
        self.conn.check_connection()
    }
}

pub struct Manager {
    client: ClusterClient,
}

impl Manager {
    pub fn new<T: IntoConnectionInfo>(nodes: Vec<T>) -> RedisResult<Self> {
        Ok(Self {
            client: ClusterClient::open(nodes)?,
        })
    }
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = ClusterConnection;
    type Error = RedisError;

    async fn create(&self) -> Result<ClusterConnection, RedisError> {
        let conn = self.client.get_connection()?;
        Ok(conn)
    }

    async fn recycle(&self, conn: &mut ClusterConnection) -> RecycleResult {
        let pong = redis::Value::Status("PONG".to_string());
        let result = conn.req_command(&redis::cmd("PING"))?;

        match result {
            redis::Value::Bulk(values) => {
                for v in values {
                    match v {
                        redis::Value::Bulk(items) => {
                            if items.len() != 2 || items[1] != pong {
                                return Err(managed::RecycleError::Message(format!(
                                    "Invalid PING response's Bulk nested element: {:?}",
                                    items
                                )));
                            }
                        }
                        e => {
                            return Err(managed::RecycleError::Message(format!(
                                "Invalid PING response's Bulk element: {:?}",
                                e
                            )))
                        }
                    }
                }
                Ok(())
            }
            v => Err(managed::RecycleError::Message(format!(
                "Invalid PING response: {:?}",
                v
            ))),
        }
    }
}
