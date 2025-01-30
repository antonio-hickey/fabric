# Fabric Client

Client for interacting with a Fabric Cache server.

- [crates.io homepage](https://crates.io/crates/fabric-cache-client)
- [documentation](https://docs.rs/fabric-cache-client/latest/fabric_cache_client/)

Install
---

Use cargo CLI:
```bash
cargo install fabric-client
```

Or manually add it into your Cargo.toml:
```toml
[dependencies]
fabric-client = "0.1.4"
```

Usage
---
For more thorough information, read the [docs](https://docs.rs/fabric-cache-client/latest/fabric_cache_client/).

Simple example for a game leaderboard cache:
```rust
use fabric_cache_client::FabricClient;
use serde::{Deserialize, Serialize};

// Dummy data structures just for example
#[derive(Deserialize, Serialize, Debug)]
struct Leaderboard {
    top_3_players: Vec<Player>,
    highest_score: i32,
    last_updated: String,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
struct Player {
    name: String,
    score: i32,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create a connection with the fabric-cache server
    let mut cache = FabricClient::connect("127.0.0.1:8731").await?;

    // Some dummy data for the example
    let players = vec![
        Player {
            name: "Leeroooy Jenkins".into(),
            score: 2830,
        },
        Player {
            name: "kinda l33t".into(),
            score: 2315,
        },
        Player {
            name: "Some Other Player".into(),
            score: 1950,
        },
    ];
    let leaderboard = Leaderboard {
        top_3_players: players.clone(),
        highest_score: players.first().map(|player| player.score).unwrap_or(0),
        last_updated: "{current timestamp}".into(),
    };

    // Insert the data into cache
    cache.set("myGamesLeaderboard", &leaderboard).await?;

    // Retrieve the data from cache
    let _leaderboard: Leaderboard = cache.get("myGamesLeaderboard").await?;

    // Update data in cache
    let mut leaderboard: Leaderboard = cache.get("myGamesLeaderboard").await?;
    leaderboard.top_3_players = vec![
        Player {
            name: "kinda l33t".into(),
            score: 2910,
        },
        Player {
            name: "Leeroooy Jenkins".into(),
            score: 2830,
        },
        Player {
            name: "Some Other Player".into(),
            score: 2100,
        },
    ];
    cache.set("myGamesLeaderboard", &leaderboard).await?;

    // Delete data in cache
    cache.remove("myGamesLeaderboard").await?;

    Ok(())
}
```
