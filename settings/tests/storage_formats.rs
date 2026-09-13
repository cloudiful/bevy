use bevy::prelude::Resource;
use bevy_persistent::{Persistent, StorageFormat};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static TEST_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Resource, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
struct StoredSettings {
    fullscreen: bool,
    ui_scale: f32,
}

#[test]
fn json_storage_format_round_trips() {
    assert_round_trip(StorageFormat::Json, "json");
}

#[test]
fn yaml_storage_format_round_trips() {
    assert_round_trip(StorageFormat::Yaml, "yaml");
}

#[test]
fn toml_storage_format_round_trips() {
    assert_round_trip(StorageFormat::Toml, "toml");
}

fn assert_round_trip(format: StorageFormat, extension: &str) {
    let path = test_file_path(extension);
    let updated = StoredSettings {
        fullscreen: true,
        ui_scale: 1.5,
    };

    let mut persistent = build_persistent(format, &path);
    assert_eq!(persistent.get(), &StoredSettings::default());
    persistent
        .update(|settings| *settings = updated.clone())
        .expect("update persisted settings");
    drop(persistent);

    let reloaded = build_persistent(format, &path);
    assert_eq!(reloaded.get(), &updated);

    std::fs::remove_file(&path).ok();
}

fn build_persistent(format: StorageFormat, path: &PathBuf) -> Persistent<StoredSettings> {
    Persistent::<StoredSettings>::builder()
        .name("storage format settings")
        .format(format)
        .path(path)
        .default(StoredSettings::default())
        .build()
        .unwrap_or_else(|err| panic!("failed to build persistent settings at {path:?}: {err}"))
}

fn test_file_path(extension: &str) -> PathBuf {
    let next_id = TEST_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "cloudiful-bevy-settings-format-{}-{next_id}.{extension}",
        std::process::id()
    ))
}
