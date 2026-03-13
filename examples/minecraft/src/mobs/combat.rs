// Combat system for mobs
// Handles damage dealing, knockback, death, and item drops

use glam::Vec3;
use rand::Rng;
use crate::inventory::Item;
use super::entity::{Entity, DeathDrop};

/// Result of a combat action
#[derive(Debug, Clone)]
pub enum CombatResult {
    /// Damage was dealt
    Damaged {
        amount: f32,
        knockback_applied: bool,
    },
    /// Entity was killed
    Killed {
        drops: Vec<ItemDrop>,
    },
    /// No damage (e.g., cooldown active)
    NoDamage,
}

/// An item that was dropped by a dead mob
#[derive(Debug, Clone)]
pub struct ItemDrop {
    pub item: Item,
    pub count: u32,
    pub position: Vec3,
}

/// Apply damage to an entity from an attacker
pub fn apply_damage(
    target: &mut Entity,
    attacker_pos: Vec3,
    damage: f32,
    knockback_strength: f32,
) -> CombatResult {
    // Try to damage (respects cooldown)
    if !target.take_damage(damage) {
        return CombatResult::NoDamage;
    }

    // Apply knockback
    let knockback_dir = (target.position - attacker_pos).normalize_or_zero();
    target.apply_knockback(knockback_dir, knockback_strength);

    CombatResult::Damaged {
        amount: damage,
        knockback_applied: true,
    }
}

/// Handle entity death and generate item drops
pub fn handle_death(
    entity: &Entity,
    death_drops: &[DeathDrop],
) -> Vec<ItemDrop> {
    if entity.is_alive() {
        return Vec::new(); // Not dead yet
    }

    let mut drops = Vec::new();
    let mut rng = rand::thread_rng();

    for drop in death_drops {
        // Check drop chance
        if rng.gen::<f32>() > drop.chance {
            continue; // Failed drop chance
        }

        // Determine count
        let count = if drop.max_count > drop.min_count {
            rng.gen_range(drop.min_count..=drop.max_count)
        } else {
            drop.min_count
        };

        if count > 0 {
            // Scatter items slightly around death position
            let scatter = Vec3::new(
                rng.gen_range(-0.3..0.3),
                0.0,
                rng.gen_range(-0.3..0.3),
            );

            drops.push(ItemDrop {
                item: drop.item,
                count,
                position: entity.position + scatter,
            });
        }
    }

    drops
}

/// Check if a mob can attack a target (range check)
pub fn can_attack(attacker_pos: Vec3, target_pos: Vec3, attack_range: f32) -> bool {
    let distance = (target_pos - attacker_pos).length();
    distance <= attack_range
}

/// Calculate knockback direction and strength based on attack
pub fn calculate_knockback(
    attacker_pos: Vec3,
    target_pos: Vec3,
    base_strength: f32,
) -> (Vec3, f32) {
    let direction = (target_pos - attacker_pos).normalize_or_zero();
    (direction, base_strength)
}

/// Combat configuration for different mob types
#[derive(Debug, Clone, Copy)]
pub struct CombatStats {
    pub attack_damage: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub knockback_strength: f32,
}

impl CombatStats {
    /// Weak melee attacker (zombie)
    pub fn zombie() -> Self {
        Self {
            attack_damage: 3.0,
            attack_range: 1.5,
            attack_cooldown: 1.0,
            knockback_strength: 2.0,
        }
    }

    /// Strong melee attacker (hypothetical)
    pub fn strong_melee() -> Self {
        Self {
            attack_damage: 6.0,
            attack_range: 2.0,
            attack_cooldown: 1.5,
            knockback_strength: 4.0,
        }
    }

    /// Weak ranged attacker (hypothetical)
    pub fn weak_ranged() -> Self {
        Self {
            attack_damage: 2.0,
            attack_range: 16.0,
            attack_cooldown: 2.0,
            knockback_strength: 1.0,
        }
    }
}

/// Damage types (for future use - resistances, armor, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Melee,
    Ranged,
    Fall,
    Fire,
    Magic,
}

/// Full damage event with metadata
#[derive(Debug, Clone)]
pub struct DamageEvent {
    pub damage: f32,
    pub damage_type: DamageType,
    pub source_position: Vec3,
    pub knockback_strength: f32,
}

impl DamageEvent {
    pub fn melee(damage: f32, source_position: Vec3) -> Self {
        Self {
            damage,
            damage_type: DamageType::Melee,
            source_position,
            knockback_strength: 2.0,
        }
    }

    pub fn fall(damage: f32, position: Vec3) -> Self {
        Self {
            damage,
            damage_type: DamageType::Fall,
            source_position: position,
            knockback_strength: 0.0, // No knockback from falling
        }
    }

    /// Apply this damage event to an entity
    pub fn apply(&self, target: &mut Entity) -> CombatResult {
        apply_damage(
            target,
            self.source_position,
            self.damage,
            self.knockback_strength,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_damage() {
        let mut target = Entity::new(Vec3::ZERO, Vec3::ONE, 10.0);
        let attacker_pos = Vec3::new(2.0, 0.0, 0.0);

        let result = apply_damage(&mut target, attacker_pos, 3.0, 2.0);

        match result {
            CombatResult::Damaged { amount, .. } => {
                assert_eq!(amount, 3.0);
                assert_eq!(target.health, 7.0);
            }
            _ => panic!("Expected damage result"),
        }
    }

    #[test]
    fn test_damage_cooldown() {
        let mut target = Entity::new(Vec3::ZERO, Vec3::ONE, 10.0);
        let attacker_pos = Vec3::ZERO;

        // First damage succeeds
        apply_damage(&mut target, attacker_pos, 3.0, 2.0);

        // Second damage fails due to cooldown
        let result = apply_damage(&mut target, attacker_pos, 3.0, 2.0);

        assert!(matches!(result, CombatResult::NoDamage));
        assert_eq!(target.health, 7.0); // Still at first damage amount
    }

    #[test]
    fn test_handle_death() {
        let mut entity = Entity::new(Vec3::ZERO, Vec3::ONE, 10.0);

        let drops_config = vec![
            DeathDrop::guaranteed(Item::RottenFlesh, 1, 2),
        ];

        // Not dead yet
        let drops = handle_death(&entity, &drops_config);
        assert!(drops.is_empty());

        // Kill the entity
        entity.health = 0.0;

        // Should drop items
        let drops = handle_death(&entity, &drops_config);
        assert!(!drops.is_empty());
        assert_eq!(drops[0].item, Item::RottenFlesh);
        assert!(drops[0].count >= 1 && drops[0].count <= 2);
    }

    #[test]
    fn test_can_attack() {
        let attacker = Vec3::ZERO;
        let target_near = Vec3::new(1.0, 0.0, 0.0);
        let target_far = Vec3::new(10.0, 0.0, 0.0);

        assert!(can_attack(attacker, target_near, 2.0));
        assert!(!can_attack(attacker, target_far, 2.0));
    }

    #[test]
    fn test_damage_event() {
        let mut target = Entity::new(Vec3::ZERO, Vec3::ONE, 10.0);

        let event = DamageEvent::melee(5.0, Vec3::new(1.0, 0.0, 0.0));
        let result = event.apply(&mut target);

        match result {
            CombatResult::Damaged { amount, .. } => {
                assert_eq!(amount, 5.0);
                assert_eq!(target.health, 5.0);
            }
            _ => panic!("Expected damage"),
        }
    }

    #[test]
    fn test_fall_damage() {
        let mut target = Entity::new(Vec3::new(0.0, 10.0, 0.0), Vec3::ONE, 10.0);

        let event = DamageEvent::fall(4.0, target.position);
        event.apply(&mut target);

        assert_eq!(target.health, 6.0);
        // Fall damage should not apply knockback
        assert_eq!(target.velocity, Vec3::ZERO);
    }

    #[test]
    fn test_combat_stats() {
        let zombie = CombatStats::zombie();
        assert_eq!(zombie.attack_damage, 3.0);
        assert_eq!(zombie.attack_range, 1.5);

        let strong = CombatStats::strong_melee();
        assert!(strong.attack_damage > zombie.attack_damage);
    }
}
