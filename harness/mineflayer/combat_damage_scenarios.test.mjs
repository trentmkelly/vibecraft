import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createCombatDamageScenariosPlan,
  summarizeCombatDamageScenarios,
  computeExpectedFallDamage,
  computeArmorMitigation,
  planMeleeAttack,
  planArrowDamageFalloff,
  planDeathMessage,
  VANILLA_DEATH_MESSAGES
} from './combat_damage_scenarios.mjs'

test('createCombatDamageScenariosPlan covers all 10 combat parity steps', () => {
  const plan = createCombatDamageScenariosPlan()
  assert.equal(plan.name, 'mineflayer-combat-damage-scenarios')
  assert.equal(plan.steps.length, 10)
  assert.ok(plan.steps.includes('melee-attack-hit-animation-damage-knockback'))
  assert.ok(plan.steps.includes('death-messages-match-vanilla'))
})

test('summarizeCombatDamageScenarios returns ok when all steps evidenced', () => {
  const plan = createCombatDamageScenariosPlan()
  const actionMap = {
    'melee-attack-hit-animation-damage-knockback': 'combat.melee.hit',
    'projectile-arrow-damage-falloff': 'combat.arrow.falloff',
    'fall-damage-formula': 'combat.fall.formula',
    'fire-damage-1-per-tick': 'combat.fire.tick',
    'drowning-damage-2-per-tick': 'combat.drown.tick',
    'void-damage-4-per-tick': 'combat.void.tick',
    'shield-blocking-projectile-negate': 'combat.shield.block',
    'armor-mitigation-formula': 'combat.armor.mitigation',
    'invulnerability-frames-0-5s': 'combat.iframes.halfSecond',
    'death-messages-match-vanilla': 'combat.death.message'
  }
  const timeline = plan.steps.map(step => ({
    name: 'combat', summary: [actionMap[step], {}]
  }))
  assert.equal(summarizeCombatDamageScenarios({ timeline }, plan).ok, true)
})

test('summarizeCombatDamageScenarios fails with empty timeline', () => {
  const plan = createCombatDamageScenariosPlan()
  assert.equal(summarizeCombatDamageScenarios({ timeline: [] }, plan).ok, false)
})

test('computeExpectedFallDamage matches vanilla floor((dist+1e-6-3)*mult)', () => {
  assert.equal(computeExpectedFallDamage(3), 0)   // safe distance -> floor(1e-6)=0
  assert.equal(computeExpectedFallDamage(3.5), 0) // fractional below 4 -> floor(0.5)=0
  assert.equal(computeExpectedFallDamage(4), 1)   // floor(1.000001)=1
  assert.equal(computeExpectedFallDamage(4.9), 1) // floor(1.900001)=1 (floors, not ceils)
  assert.equal(computeExpectedFallDamage(7), 4)   // floor(4.000001)=4
  assert.equal(computeExpectedFallDamage(0), 0)   // clamped to >=0
})

test('computeArmorMitigation reduces damage per armor formula', () => {
  // Iron chestplate: 8 armor, 0 toughness, 10 damage
  const ironResult = computeArmorMitigation(8, 0, 10)
  assert.ok(ironResult > 0 && ironResult < 10, `iron mitigation: ${ironResult}`)

  // Full diamond: 20 armor, 8 toughness, 10 damage → reduces more
  const diamondResult = computeArmorMitigation(20, 8, 10)
  assert.ok(diamondResult < ironResult, 'diamond should reduce more than iron')
})

test('planMeleeAttack and planArrowDamageFalloff produce correct shapes', () => {
  const melee = planMeleeAttack('Attacker', 'Target', 6.0, { x: 0.3, z: 0.3 })
  assert.equal(melee.action, 'combat.melee.hit')
  assert.equal(melee.expectedDamage, 6.0)

  const arrow = planArrowDamageFalloff(30, [1, 5])
  assert.equal(arrow.action, 'combat.arrow.falloff')
  assert.equal(arrow.distance, 30)
})

test('VANILLA_DEATH_MESSAGES covers all required sources', () => {
  const required = ['fall', 'fire', 'drown', 'suffocation', 'void', 'mob_attack', 'player_attack', 'arrow', 'fireball', 'tnt']
  for (const source of required) {
    assert.ok(VANILLA_DEATH_MESSAGES[source], `missing death message for ${source}`)
    assert.ok(VANILLA_DEATH_MESSAGES[source].startsWith('death.attack.') || VANILLA_DEATH_MESSAGES[source].startsWith('death.fell.'))
  }
})

test('planDeathMessage records cause and expected key', () => {
  const p = planDeathMessage('minecraft:fall', 'death.attack.fall')
  assert.equal(p.causeSource, 'minecraft:fall')
  assert.equal(p.expectedMessageKey, 'death.attack.fall')
})
