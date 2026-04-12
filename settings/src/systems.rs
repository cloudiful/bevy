use crate::{RequestedSettingAction, SettingActionHandler};
use bevy::{
    app::{App, Update},
    ecs::{
        message::{Message, MessageReader},
        schedule::{IntoScheduleConfigs, SystemSet},
        system::{ResMut, ScheduleSystem},
    },
    prelude::Resource,
};
use bevy_persistent::Persistent;
use serde::{Serialize, de::DeserializeOwned};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingSystemSet {
    ApplyActions,
    SyncUi,
}

pub fn apply_setting_action<T, Action, AppAction>(settings: &mut Persistent<T>, action: AppAction)
where
    T: Resource + Serialize + DeserializeOwned + SettingActionHandler<Action>,
    Action: Copy + TryFrom<AppAction>,
    AppAction: Copy,
{
    let Ok(action) = Action::try_from(action) else {
        return;
    };

    if !SettingActionHandler::can_apply(settings.get(), action) {
        return;
    }

    settings
        .update(|setting: &mut T| {
            SettingActionHandler::apply(setting, action);
        })
        .unwrap();
}

pub fn change_setting<T, Action, Requested, AppAction>(
    mut settings: ResMut<Persistent<T>>,
    mut action_messages: MessageReader<Requested>,
) where
    T: Resource + Serialize + DeserializeOwned + SettingActionHandler<Action>,
    Action: Copy + TryFrom<AppAction>,
    Requested: Message + RequestedSettingAction<AppAction>,
    AppAction: Copy,
{
    for message in action_messages.read() {
        apply_setting_action(&mut settings, message.action());
    }
}

pub fn register_setting_systems<ChangeMarker, SyncMarker>(
    app: &mut App,
    change_systems: impl IntoScheduleConfigs<ScheduleSystem, ChangeMarker>,
    sync_systems: impl IntoScheduleConfigs<ScheduleSystem, SyncMarker>,
) -> &mut App {
    app.configure_sets(
        Update,
        (
            SettingSystemSet::ApplyActions,
            SettingSystemSet::SyncUi.after(SettingSystemSet::ApplyActions),
        ),
    )
    .add_systems(
        Update,
        change_systems.in_set(SettingSystemSet::ApplyActions),
    )
    .add_systems(Update, sync_systems.in_set(SettingSystemSet::SyncUi))
}

#[cfg(test)]
mod tests {
    use super::{apply_setting_action, change_setting};
    use crate::{RequestedSettingAction, SettingActionHandler};
    use bevy::{app::App, ecs::message::Message, prelude::*};
    use bevy_persistent::{Persistent, StorageFormat};
    use serde::{Deserialize, Serialize};
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static TEST_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestAction {
        Increase,
        Disabled,
    }

    #[derive(Resource, Debug, Default, Serialize, Deserialize)]
    struct TestSettings {
        value: u32,
        applied: u32,
    }

    impl SettingActionHandler<TestAction> for TestSettings {
        fn can_apply(&self, action: TestAction) -> bool {
            match action {
                TestAction::Increase => self.value == 0,
                TestAction::Disabled => false,
            }
        }

        fn apply(&mut self, action: TestAction) {
            if let TestAction::Increase = action {
                self.value += 1;
                self.applied += 1;
            }
        }
    }

    #[derive(Message, Debug, Clone, Copy)]
    struct TestRequested {
        action: TestAction,
    }

    impl RequestedSettingAction<TestAction> for TestRequested {
        fn action(&self) -> TestAction {
            self.action
        }
    }

    #[test]
    fn apply_setting_action_respects_can_apply_and_updates_once() {
        let mut settings = test_persistent(TestSettings::default());

        apply_setting_action::<_, TestAction, TestAction>(&mut settings, TestAction::Disabled);
        assert_eq!(settings.get().value, 0);
        assert_eq!(settings.get().applied, 0);

        apply_setting_action::<_, TestAction, TestAction>(&mut settings, TestAction::Increase);
        assert_eq!(settings.get().value, 1);
        assert_eq!(settings.get().applied, 1);

        apply_setting_action::<_, TestAction, TestAction>(&mut settings, TestAction::Increase);
        assert_eq!(settings.get().value, 1);
        assert_eq!(settings.get().applied, 1);
    }

    #[test]
    fn change_setting_reads_action_from_requested_message() {
        let mut app = App::new();
        app.add_message::<TestRequested>();
        app.insert_resource(test_persistent(TestSettings::default()));
        app.add_systems(
            Update,
            change_setting::<TestSettings, TestAction, TestRequested, TestAction>,
        );

        app.world_mut().write_message(TestRequested {
            action: TestAction::Increase,
        });
        app.update();

        let settings = app.world().resource::<Persistent<TestSettings>>();
        assert_eq!(settings.get().value, 1);
        assert_eq!(settings.get().applied, 1);
    }

    fn test_persistent(default: TestSettings) -> Persistent<TestSettings> {
        let path = test_file_path("settings");

        Persistent::<TestSettings>::builder()
            .name("test settings")
            .format(StorageFormat::Toml)
            .path(&path)
            .default(default)
            .build()
            .unwrap_or_else(|err| panic!("failed to build persistent settings at {path:?}: {err}"))
    }

    fn test_file_path(name: &str) -> PathBuf {
        let next_id = TEST_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "cloudiful-bevy-settings-{name}-{}-{next_id}.toml",
            std::process::id()
        ))
    }
}
