#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    Read,
    Close,
    ReloadSettings,
    OpenSettings,
    ToggleWindowVisible,
    PlayPause,
    RateDown,
    RateUp,
}

pub const ACTION_LIST: [Action; 8] = [
    Action::Read,
    Action::Close,
    Action::ReloadSettings,
    Action::OpenSettings,
    Action::ToggleWindowVisible,
    Action::PlayPause,
    Action::RateDown,
    Action::RateUp,
];

#[test]
fn action_list_match_enum() {
    for (id, &act) in ACTION_LIST.iter().enumerate() {
        assert_eq!(id, act as usize);
    }
}

#[test]
fn action_list_match_settings() {
    assert_eq!(ACTION_LIST.len(), ::Settings::new().hotkeys.len());
}

impl ::std::fmt::Display for Action {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::Action::*;
        match self {
            &Read => write!(f, "read"),
            &Close => write!(f, "close"),
            &ReloadSettings => write!(f, "reload_settings"),
            &OpenSettings => write!(f, "open_settings"),
            &ToggleWindowVisible => write!(f, "toggle_window_visible"),
            &PlayPause => write!(f, "play_pause"),
            &RateDown => write!(f, "rate_down"),
            &RateUp => write!(f, "rate_up"),
        }
    }
}
