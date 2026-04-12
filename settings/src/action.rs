pub trait SettingActionHandler<Action: Copy> {
    fn can_apply(&self, action: Action) -> bool;
    fn apply(&mut self, action: Action);
}

pub trait RequestedSettingAction<Action> {
    fn action(&self) -> Action;
}
