/// Tracks the player's money and handles purchasing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Economy {
    pub money: i32,
}

impl Economy {
    /// Creates a new `Economy` instance with the specified starting money.
    pub fn new(money: i32) -> Self {
        Self { money }
    }

    /// Checks if the player can afford a specific cost.
    pub fn can_afford(&self, cost: i32) -> bool {
        self.money >= cost
    }

    /// Attempts to deduct the cost from the player's money.
    /// Returns `true` if successful (funds were sufficient and deducted), or `false` otherwise.
    pub fn purchase(&mut self, cost: i32) -> bool {
        if self.can_afford(cost) {
            self.money -= cost;
            true
        } else {
            false
        }
    }

    /// Adds the specified amount of money to the player's economy.
    pub fn add_money(&mut self, amount: i32) {
        self.money += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purchase_succeeds_with_enough_money() {
        let mut economy = Economy::new(100);
        assert!(economy.can_afford(50));
        assert!(economy.purchase(50));
        assert_eq!(economy.money, 50);
    }

    #[test]
    fn test_purchase_fails_with_insufficient_funds() {
        let mut economy = Economy::new(30);
        assert!(!economy.can_afford(50));
        assert!(!economy.purchase(50));
        assert_eq!(economy.money, 30);
    }

    #[test]
    fn test_money_updates_correctly() {
        let mut economy = Economy::new(100);
        economy.add_money(50);
        assert_eq!(economy.money, 150);

        assert!(economy.purchase(120));
        assert_eq!(economy.money, 30);
    }
}
