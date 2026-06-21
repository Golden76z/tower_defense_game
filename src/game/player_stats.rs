/// Tracks player lives and game over status.
pub struct PlayerStats {
    pub lives: i32,
    pub max_lives: i32,
}

impl PlayerStats {
    /// Creates a new player stats tracker with the given initial lives.
    pub fn new(lives: i32) -> Self {
        Self {
            lives,
            max_lives: lives,
        }
    }

    /// Reduces player lives by the given amount, clamping to a minimum of 0.
    pub fn take_damage(&mut self, amount: i32) {
        self.lives = (self.lives - amount).max(0);
    }

    /// Checks if the player has any lives remaining.
    pub fn is_alive(&self) -> bool {
        self.lives > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lives_decrease_on_damage() {
        let mut stats = PlayerStats::new(5);
        assert_eq!(stats.lives, 5);
        assert!(stats.is_alive());

        stats.take_damage(2);
        assert_eq!(stats.lives, 3);
        assert!(stats.is_alive());
    }

    #[test]
    fn test_game_over_when_lives_reach_0() {
        let mut stats = PlayerStats::new(5);
        assert!(stats.is_alive());

        stats.take_damage(5);
        assert_eq!(stats.lives, 0);
        assert!(!stats.is_alive());

        // Further damage shouldn't make lives negative
        stats.take_damage(1);
        assert_eq!(stats.lives, 0);
    }
}
