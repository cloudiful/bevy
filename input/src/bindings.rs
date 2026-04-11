use bevy::input::gamepad::{GamepadAxis, GamepadButton};
use bevy::input::keyboard::KeyCode;
use bevy::prelude::Resource;
use std::collections::HashMap;
use std::hash::Hash;

pub trait InputAction: Copy + Eq + Hash + Send + Sync + 'static {}

impl<T> InputAction for T where T: Copy + Eq + Hash + Send + Sync + 'static {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputButton {
    Key(KeyCode),
    Gamepad(GamepadButton),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputBinding {
    Button(InputButton),
    GamepadAxis(GamepadAxis),
    ButtonAxis {
        negative: InputButton,
        positive: InputButton,
    },
}

#[derive(Resource, Debug, Clone)]
pub struct InputMap<A: InputAction> {
    bindings: HashMap<A, Vec<InputBinding>>,
}

impl<A: InputAction> Default for InputMap<A> {
    fn default() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
}

impl<A: InputAction> InputMap<A> {
    pub fn bindings(&self, action: A) -> &[InputBinding] {
        self.bindings.get(&action).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn actions(&self) -> impl Iterator<Item = A> + '_ {
        self.bindings.keys().copied()
    }

    pub fn clear_action(&mut self, action: A) {
        self.bindings.remove(&action);
    }

    pub fn set_bindings(
        &mut self,
        action: A,
        bindings: impl IntoIterator<Item = InputBinding>,
    ) -> &mut Self {
        self.bindings.insert(action, bindings.into_iter().collect());
        self
    }

    pub fn bind_key(&mut self, action: A, key: KeyCode) -> &mut Self {
        self.push(action, InputBinding::Button(InputButton::Key(key)))
    }

    pub fn bind_gamepad_button(&mut self, action: A, button: GamepadButton) -> &mut Self {
        self.push(action, InputBinding::Button(InputButton::Gamepad(button)))
    }

    pub fn bind_gamepad_axis(&mut self, action: A, axis: GamepadAxis) -> &mut Self {
        self.push(action, InputBinding::GamepadAxis(axis))
    }

    pub fn bind_button_axis(
        &mut self,
        action: A,
        negative: InputButton,
        positive: InputButton,
    ) -> &mut Self {
        self.push(action, InputBinding::ButtonAxis { negative, positive })
    }

    pub fn rebind_button(&mut self, action: A, old: InputButton, new: InputButton) -> bool {
        self.replace(action, |binding| match binding {
            InputBinding::Button(button) if *button == old => {
                *binding = InputBinding::Button(new);
                true
            }
            InputBinding::ButtonAxis { negative, positive } if *negative == old => {
                *binding = InputBinding::ButtonAxis {
                    negative: new,
                    positive: *positive,
                };
                true
            }
            InputBinding::ButtonAxis { negative, positive } if *positive == old => {
                *binding = InputBinding::ButtonAxis {
                    negative: *negative,
                    positive: new,
                };
                true
            }
            _ => false,
        })
    }

    pub fn rebind_gamepad_axis(&mut self, action: A, old: GamepadAxis, new: GamepadAxis) -> bool {
        self.replace(action, |binding| match binding {
            InputBinding::GamepadAxis(axis) if *axis == old => {
                *binding = InputBinding::GamepadAxis(new);
                true
            }
            _ => false,
        })
    }

    fn push(&mut self, action: A, binding: InputBinding) -> &mut Self {
        self.bindings.entry(action).or_default().push(binding);
        self
    }

    fn replace(&mut self, action: A, mut replace: impl FnMut(&mut InputBinding) -> bool) -> bool {
        let Some(bindings) = self.bindings.get_mut(&action) else {
            return false;
        };

        for binding in bindings.iter_mut() {
            if replace(binding) {
                return true;
            }
        }

        false
    }
}
