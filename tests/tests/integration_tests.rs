use fabric_cache_client::FabricClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{process::Command, thread};

#[tokio::test]
async fn can_start_and_connect_to_server() {
    thread::spawn(move || {
        Command::new("./../target/debug/fabric-cache")
            .output()
            .expect("Failed to start Fabric Cache Server");
    });

    std::thread::sleep(std::time::Duration::from_secs(1));

    assert!(FabricClient::connect("127.0.0.1:8731").await.is_ok())
}

#[tokio::test]
async fn can_handle_basic_use() {
    thread::spawn(move || {
        Command::new("./../target/debug/fabric-cache")
            .output()
            .expect("Failed to start Fabric Cache Server");
    });

    std::thread::sleep(std::time::Duration::from_secs(1));

    let mut client = FabricClient::connect("127.0.0.1:8731").await.unwrap();

    // Test data, 3D coordinate data structs
    let coordinates: HashMap<String, ThreeDimensionalCoordinate> = HashMap::from([
        (
            "House".into(),
            ThreeDimensionalCoordinate { x: 10, y: 29, z: 6 },
        ),
        (
            "School".into(),
            ThreeDimensionalCoordinate { x: 87, y: 61, z: 9 },
        ),
        (
            "Park".into(),
            ThreeDimensionalCoordinate { x: 74, y: 2, z: 3 },
        ),
        (
            "Store".into(),
            ThreeDimensionalCoordinate { x: 3, y: 33, z: 12 },
        ),
    ]);

    // Add the test coordinate structs to the "coordinates" cache
    client
        .set("coordinates", &coordinates)
        .await
        .expect("Could not set coordinates value");

    // Loop over each data point
    for key in coordinates.keys() {
        let cache_key = format!("coordinates.{}", key);

        let expected_coordinates = coordinates.get(key).unwrap();
        let actual_coordinates: ThreeDimensionalCoordinate =
            serde_json::from_value(client.get(&cache_key).await.unwrap()).unwrap();

        // Grab the coordinates from cache and verify they're as expected
        assert_eq!(expected_coordinates.x, actual_coordinates.x);
        assert_eq!(expected_coordinates.y, actual_coordinates.y);
        assert_eq!(expected_coordinates.z, actual_coordinates.z);

        // Change the x coordinate and regrab it to make sure it was updated
        client.set(&format!("{}.x", cache_key), &12).await.unwrap();
        let actual_coordinates: ThreeDimensionalCoordinate =
            serde_json::from_value(client.get(&cache_key).await.unwrap()).unwrap();
        assert_eq!(12, actual_coordinates.x);
    }
}

#[tokio::test]
async fn can_handle_nested_structs() {
    thread::spawn(move || {
        Command::new("./../target/debug/fabric-cache")
            .output()
            .expect("Failed to start Fabric Cache Server");
    });

    std::thread::sleep(std::time::Duration::from_secs(1));

    let mut client = FabricClient::connect("127.0.0.1:8731").await.unwrap();

    // Nested test data, user data structs
    let users: HashMap<String, NestedTestStruct> = HashMap::from([
        (
            "user_1".into(),
            NestedTestStruct {
                id: "user_1".into(),
                age: 21,
                billing: BillingTestStruct {
                    city: "Langley".into(),
                    state: "Virginia".into(),
                    address: "123 Some Rd".into(),
                },
                settings: SettingsTestStruct {
                    auto_play: false,
                    max_replays: 2,
                    default_volume: 88.50,
                },
            },
        ),
        (
            "user_2".into(),
            NestedTestStruct {
                id: "user_2".into(),
                age: 28,
                billing: BillingTestStruct {
                    city: "Miami".into(),
                    state: "Florida".into(),
                    address: "334 Alligator St E".into(),
                },
                settings: SettingsTestStruct {
                    auto_play: true,
                    max_replays: 1,
                    default_volume: 100.00,
                },
            },
        ),
    ]);

    // Add the test data structs to the "users" cache
    client
        .set("users", &users)
        .await
        .expect("Could not set coordinates value");

    for key in users.keys() {
        let cache_key = format!("users.{}", key);

        let expected_user = users.get(key).unwrap();
        let actual_user: NestedTestStruct =
            serde_json::from_value(client.get(&cache_key).await.unwrap()).unwrap();

        assert_eq!(expected_user.id, actual_user.id);
        assert_eq!(expected_user.age, actual_user.age);
        assert_eq!(
            expected_user.settings.auto_play,
            actual_user.settings.auto_play
        );
        assert_eq!(
            expected_user.settings.default_volume,
            actual_user.settings.default_volume
        );
        assert_eq!(
            expected_user.settings.max_replays,
            actual_user.settings.max_replays,
        );
        assert_eq!(expected_user.billing.address, actual_user.billing.address);
        assert_eq!(expected_user.billing.state, actual_user.billing.state);
        assert_eq!(expected_user.billing.city, actual_user.billing.city);

        // Change the user's address and regrab it to make sure the cache was updated
        // NOTE: we do a whole struct update here with the `BillingTestStruct`
        let new_billing_info = BillingTestStruct {
            address: "321 Some New Rd".into(),
            city: "Some New City".into(),
            state: "Utah".into(),
        };
        client
            .set(&format!("{}.billing", &cache_key), &new_billing_info)
            .await
            .unwrap();
        let actual_user: NestedTestStruct =
            serde_json::from_value(client.get(&cache_key).await.unwrap()).unwrap();
        assert_eq!(new_billing_info.address, actual_user.billing.address);
        assert_eq!(new_billing_info.state, actual_user.billing.state);
        assert_eq!(new_billing_info.city, actual_user.billing.city);
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct ThreeDimensionalCoordinate {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Deserialize, Serialize, Debug)]
struct NestedTestStruct {
    id: String,
    age: i32,
    billing: BillingTestStruct,
    settings: SettingsTestStruct,
}
#[derive(Deserialize, Serialize, Debug)]
struct BillingTestStruct {
    address: String,
    city: String,
    state: String,
}
#[derive(Deserialize, Serialize, Debug)]
struct SettingsTestStruct {
    auto_play: bool,
    max_replays: i32,
    default_volume: f64,
}
