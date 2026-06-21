#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
    Victory,
}

impl Default for GameState {
    fn default() -> Self {
        Self::MainMenu
    }
}

impl GameState {
    pub fn transition_on_escape(self) -> Self {
        match self {
            Self::Playing => Self::Paused,
            Self::Paused => Self::Playing,
            _ => self,
        }
    }

    pub fn transition_on_start(self) -> Self {
        match self {
            Self::MainMenu | Self::GameOver | Self::Victory => Self::Playing,
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        assert_eq!(GameState::default(), GameState::MainMenu);
    }

    #[test]
    fn test_transition_on_start() {
        assert_eq!(GameState::MainMenu.transition_on_start(), GameState::Playing);
        assert_eq!(GameState::GameOver.transition_on_start(), GameState::Playing);
        assert_eq!(GameState::Victory.transition_on_start(), GameState::Playing);
        assert_eq!(GameState::Playing.transition_on_start(), GameState::Playing);
    }

    #[test]
    fn test_transition_on_escape() {
        assert_eq!(GameState::Playing.transition_on_escape(), GameState::Paused);
        assert_eq!(GameState::Paused.transition_on_escape(), GameState::Playing);
        assert_eq!(GameState::MainMenu.transition_on_escape(), GameState::MainMenu);
    }
}
