// Player health system
// Manages health, damage, death state, respawning, and damage cooldown

use std::time::{Duration, Instant};

/// Health system constants
pub const MAX_HEALTH: f32 = 20.0;
pub const DAMAGE_COOLDOWN: Duration = Duration::from_millis(500); // 0.5 seconds invulnerability after hit

/// Player health state
pub struct Health {
    current: f32,
    max: f32,
    last_damage_time: Option<Instant>,
    is_dead: bool,
}

impl Health {
    /// Create a new health system with the given maximum health
    pub fn new(max_health: f32) -> Self {
        Self {
            current: max_health,
            max: max_health,
            last_damage_time: None,
            is_dead: false,
        }
    }

    /// Get current health
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Get maximum health
    pub fn max(&self) -> f32 {
        self.max
    }

    /// Get health as a percentage (0.0 to 1.0)
    pub fn percentage(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }

    /// Check if player is dead
    pub fn is_dead(&self) -> bool {
        self.is_dead
    }

    /// Check if player is alive (opposite of is_dead)
    pub fn is_alive(&self) -> bool {
        !self.is_dead
    }

    /// Check if player is at full health
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Apply damage to the player
    /// Returns true if damage was applied, false if on cooldown
    pub fn damage(&mut self, amount: f32) -> bool {
        if self.is_dead || amount <= 0.0 {
            return false;
        }

        // Check damage cooldown
        if let Some(last_time) = self.last_damage_time {
            if last_time.elapsed() < DAMAGE_COOLDOWN {
                return false; // Still on cooldown
            }
        }

        // Apply damage
        self.current -= amount;
        self.last_damage_time = Some(Instant::now());

        // Check for death
        if self.current <= 0.0 {
            self.current = 0.0;
            self.is_dead = true;
        }

        true
    }

    /// Alias for damage() - take damage from an external source
    pub fn take_damage(&mut self, amount: i32) -> bool {
        self.damage(amount as f32)
    }

    /// Heal the player
    /// Returns the actual amount healed
    pub fn heal(&mut self, amount: f32) -> f32 {
        if self.is_dead || amount <= 0.0 {
            return 0.0;
        }

        let old_health = self.current;
        self.current = (self.current + amount).min(self.max);
        self.current - old_health
    }

    /// Restore to full health
    pub fn restore(&mut self) {
        if !self.is_dead {
            self.current = self.max;
        }
    }

    /// Respawn the player (restore to full health and clear death state)
    pub fn respawn(&mut self) {
        self.current = self.max;
        self.is_dead = false;
        self.last_damage_time = None;
    }

    /// Set the current health to a specific value
    pub fn set_health(&mut self, health: i32) {
        self.current = (health as f32).clamp(0.0, self.max);
        self.is_dead = self.current <= 0.0;
    }

    /// Set maximum health (and optionally restore current health to max)
    pub fn set_max(&mut self, new_max: f32, restore: bool) {
        self.max = new_max.max(1.0);
        if restore {
            self.current = self.max;
        } else {
            self.current = self.current.min(self.max);
        }
    }

    /// Get time remaining on damage cooldown
    pub fn cooldown_remaining(&self) -> Duration {
        if let Some(last_time) = self.last_damage_time {
            let elapsed = last_time.elapsed();
            if elapsed < DAMAGE_COOLDOWN {
                return DAMAGE_COOLDOWN - elapsed;
            }
        }
        Duration::ZERO
    }

    /// Check if damage cooldown is active
    pub fn is_on_cooldown(&self) -> bool {
        self.cooldown_remaining() > Duration::ZERO
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(MAX_HEALTH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_health_creation() {
        let health = Health::new(20.0);
        assert_eq!(health.current(), 20.0);
        assert_eq!(health.max(), 20.0);
        assert!(!health.is_dead());
        assert!(health.is_full());
    }

    #[test]
    fn test_damage() {
        let mut health = Health::new(20.0);

        // Apply damage
        let applied = health.damage(5.0);
        assert!(applied);
        assert_eq!(health.current(), 15.0);
        assert!(!health.is_dead());
    }

    #[test]
    fn test_death() {
        let mut health = Health::new(20.0);

        // Apply lethal damage
        health.damage(25.0);
        assert_eq!(health.current(), 0.0);
        assert!(health.is_dead());

        // Can't damage when dead
        let applied = health.damage(5.0);
        assert!(!applied);
        assert_eq!(health.current(), 0.0);
    }

    #[test]
    fn test_healing() {
        let mut health = Health::new(20.0);
        health.damage(10.0);
        assert_eq!(health.current(), 10.0);

        // Heal partially
        let healed = health.heal(5.0);
        assert_eq!(healed, 5.0);
        assert_eq!(health.current(), 15.0);

        // Heal beyond max (should clamp)
        let healed = health.heal(10.0);
        assert_eq!(healed, 5.0);
        assert_eq!(health.current(), 20.0);
        assert!(health.is_full());
    }

    #[test]
    fn test_respawn() {
        let mut health = Health::new(20.0);

        // Kill player
        health.damage(25.0);
        assert!(health.is_dead());
        assert_eq!(health.current(), 0.0);

        // Respawn
        health.respawn();
        assert!(!health.is_dead());
        assert_eq!(health.current(), 20.0);
    }

    #[test]
    fn test_damage_cooldown() {
        let mut health = Health::new(20.0);

        // First damage should apply
        let applied = health.damage(5.0);
        assert!(applied);
        assert_eq!(health.current(), 15.0);

        // Immediate second damage should be blocked
        let applied = health.damage(5.0);
        assert!(!applied);
        assert_eq!(health.current(), 15.0);

        // Wait for cooldown
        sleep(Duration::from_millis(550));

        // Damage should apply again
        let applied = health.damage(5.0);
        assert!(applied);
        assert_eq!(health.current(), 10.0);
    }

    #[test]
    fn test_percentage() {
        let mut health = Health::new(20.0);
        assert_eq!(health.percentage(), 1.0);

        health.damage(10.0);
        assert_eq!(health.percentage(), 0.5);

        health.damage(10.0);
        assert_eq!(health.percentage(), 0.0);
    }

    #[test]
    fn test_set_max() {
        let mut health = Health::new(20.0);

        // Reduce max without restoring
        health.set_max(10.0, false);
        assert_eq!(health.max(), 10.0);
        assert_eq!(health.current(), 10.0); // Current clamped to new max

        // Increase max without restoring
        health.set_max(20.0, false);
        assert_eq!(health.max(), 20.0);
        assert_eq!(health.current(), 10.0); // Current unchanged

        // Increase max and restore
        health.set_max(30.0, true);
        assert_eq!(health.max(), 30.0);
        assert_eq!(health.current(), 30.0); // Current restored to max
    }

    #[test]
    fn test_no_damage_when_dead() {
        let mut health = Health::new(20.0);
        health.damage(20.0);
        assert!(health.is_dead());

        // Try to heal while dead (should fail)
        let healed = health.heal(10.0);
        assert_eq!(healed, 0.0);
        assert_eq!(health.current(), 0.0);
        assert!(health.is_dead());
    }

    #[test]
    fn test_restore() {
        let mut health = Health::new(20.0);
        health.damage(15.0);
        assert_eq!(health.current(), 5.0);

        health.restore();
        assert_eq!(health.current(), 20.0);
        assert!(health.is_full());
    }
}
