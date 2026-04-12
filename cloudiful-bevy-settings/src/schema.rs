#[derive(Debug, Clone)]
pub struct SettingFieldSpec<AppAction, FieldKey, TextKey> {
    pub label: String,
    pub label_key: Option<TextKey>,
    pub control: SettingControlSpec<AppAction, FieldKey>,
}

#[derive(Debug, Clone)]
pub struct SettingSelectOption<Action> {
    pub text: String,
    pub action: Action,
    pub disabled: bool,
}

#[derive(Debug, Clone)]
pub struct SettingSliderSpec<Action> {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub precision: i32,
    pub suffix: String,
    pub action: fn(f32) -> Action,
}

impl<Action> SettingSliderSpec<Action> {
    pub fn new(
        value: f32,
        min: f32,
        max: f32,
        step: f32,
        precision: i32,
        suffix: impl Into<String>,
        action: fn(f32) -> Action,
    ) -> Self {
        Self {
            value,
            min,
            max,
            step,
            precision,
            suffix: suffix.into(),
            action,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SettingControlSpec<Action, FieldKey> {
    Toggle {
        text: String,
        action: Action,
    },
    Stepper {
        key: FieldKey,
        value: String,
        decrease_action: Option<Action>,
        increase_action: Option<Action>,
        slider: Option<SettingSliderSpec<Action>>,
    },
    Select {
        key: FieldKey,
        value: String,
        options: Vec<SettingSelectOption<Action>>,
    },
}

impl<Action, FieldKey, TextKey> SettingFieldSpec<Action, FieldKey, TextKey> {
    pub fn toggle(label: impl Into<String>, text: String, action: Action) -> Self {
        Self {
            label: label.into(),
            label_key: None,
            control: SettingControlSpec::Toggle { text, action },
        }
    }

    pub fn stepper(
        label: impl Into<String>,
        key: FieldKey,
        value: String,
        decrease_action: Option<Action>,
        increase_action: Option<Action>,
    ) -> Self {
        Self {
            label: label.into(),
            label_key: None,
            control: SettingControlSpec::Stepper {
                key,
                value,
                decrease_action,
                increase_action,
                slider: None,
            },
        }
    }

    pub fn select(
        label: impl Into<String>,
        key: FieldKey,
        value: String,
        options: Vec<SettingSelectOption<Action>>,
    ) -> Self {
        Self {
            label: label.into(),
            label_key: None,
            control: SettingControlSpec::Select {
                key,
                value,
                options,
            },
        }
    }

    pub fn with_label_key(mut self, label_key: TextKey) -> Self {
        self.label_key = Some(label_key);
        self
    }

    pub fn with_slider(mut self, slider: SettingSliderSpec<Action>) -> Self {
        if let SettingControlSpec::Stepper {
            slider: slider_slot,
            ..
        } = &mut self.control
        {
            *slider_slot = Some(slider);
        }
        self
    }
}

pub trait SettingFieldSource<AppAction, Context, FieldKey, TextKey> {
    fn field_specs(&self, context: &Context)
    -> Vec<SettingFieldSpec<AppAction, FieldKey, TextKey>>;
}

#[cfg(test)]
mod tests {
    use super::SettingSliderSpec;

    fn slider_action(value: f32) -> u32 {
        value.round() as u32
    }

    #[test]
    fn slider_spec_new_preserves_fields() {
        let spec = SettingSliderSpec::new(42.0, 0.0, 100.0, 5.0, 1, "%", slider_action);

        assert_eq!(spec.value, 42.0);
        assert_eq!(spec.min, 0.0);
        assert_eq!(spec.max, 100.0);
        assert_eq!(spec.step, 5.0);
        assert_eq!(spec.precision, 1);
        assert_eq!(spec.suffix, "%");
        assert_eq!((spec.action)(12.4), 12);
    }
}
