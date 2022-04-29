# Deadpool for Redis Cluster

Deadpool is a dead simple async pool for connections and objects of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool) manager for [`redis`](https://crates.io/crates/redis) Cluster mode.

## Example

```rust
use deadpool_redis_cluster::{redis::Commands, Config, Runtime};

#[tokio::main]
async fn main() {
    let nodes = vec![
        "redis://127.0.0.1:6379/",
        "redis://127.0.0.1:6378/",
        "redis://127.0.0.1:6377/",
    ];
    let cfg = Config::from_nodes(nodes);
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
    {
        let mut con = pool.get().await.unwrap();
        let _: () = con.set("test", "test_data").unwrap();
    }
    {
        let mut con = pool.get().await.unwrap();
        let rv: String = con.get("test").unwrap();

        assert_eq!(rv, "test_data");
    }
}
```
