use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Entity {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct World(Vec<Entity>);

use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(with = "humantime_serde")]
    timeout: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serde() {
        let world = World(vec![Entity {
            x: 1. / 3.,
            y: 2. / 3.,
        }]);

        let json = serde_json::to_string(&world).unwrap();
        let deserialized: World = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, world);
    }

    #[test]
    fn test_serde_bincode() {
        let world = World(vec![Entity {
            x: 1. / 3.,
            y: 2. / 3.,
        }]);

        let json = serde_json::to_string(&world).unwrap();
        let deserialized: World = serde_json::from_str(&json).unwrap();

        let encoded: Vec<u8> = bincode::serialize(&world).unwrap();
        let decoded: World = bincode::deserialize(&encoded).unwrap();

        assert_eq!(json.len(), 32);
        assert_eq!(encoded.len(), 16);

        assert_eq!(deserialized, decoded);
    }

    #[test]
    fn test_human_time() {
        let config: Config = serde_json::from_str(r#"{"timeout":"3m"}"#).unwrap();
        assert_eq!(config.timeout, Duration::from_secs(180));
    }
}
