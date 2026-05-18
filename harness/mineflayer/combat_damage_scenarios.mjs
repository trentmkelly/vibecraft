// Combat/damage parity tests: melee attack, projectile arrow, fall damage,
// fire damage, drowning damage, void damage, shield blocking, armor mitigation,
// invulnerability frames, vanilla-compatible damage/death messages.

export function createCombatDamageScenariosPlan(options = {}) {
  return {
    name: 'mineflayer-combat-damage-scenarios',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'CombatBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'normal',
      'spawn-protection': '0'
    },
    steps: [
      'melee-attack-hit-animation-damage-knockback',
      'projectile-arrow-damage-falloff',
      'fall-damage-formula',
      'fire-damage-1-per-tick',
      'drowning-damage-2-per-tick',
      'void-damage-4-per-tick',
      'shield-blocking-projectile-negate',
      'armor-mitigation-formula',
      'invulnerability-frames-0-5s',
      'death-messages-match-vanilla'
    ]
  }
}

export function summarizeCombatDamageScenarios(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => e.name === 'combat')
    .map(e => e.summary?.[0])
  const result = Object.fromEntries(
    plan.steps.map(step => [step, actions.includes(stepToAction(step))])
  )
  return {
    ok: Object.values(result).every(Boolean),
    steps: result
  }
}

function stepToAction(step) {
  const map = {
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
  return map[step] ?? step
}

export function recordCombatEvent(session, action, details = {}) {
  session.timeline.push({ name: 'combat', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

// Fall damage formula: max(0, ceil(fallDistance - 3)) * 1.0 HP (no multiplier in 1.21)
export function computeExpectedFallDamage(fallDistance) {
  return Math.max(0, Math.ceil(fallDistance - 3))
}

// Armor mitigation formula: max(0, ceil(armor * 0.04 * rawDamage)) as percentage
export function computeArmorMitigation(armorPoints, toughness, rawDamage) {
  const effectiveArmor = Math.max(armorPoints * 0.2, Math.min(20, armorPoints - rawDamage / (2 + toughness / 4)))
  return rawDamage * (1 - effectiveArmor / 25)
}

export function planMeleeAttack(attackerName, targetName, expectedDamage, expectedKnockback) {
  return { action: 'combat.melee.hit', attackerName, targetName, expectedDamage, expectedKnockback }
}

export function planArrowDamageFalloff(distance, expectedDamageRange) {
  return { action: 'combat.arrow.falloff', distance, expectedDamageRange }
}

export function planDeathMessage(causeSource, expectedMessageKey) {
  return { action: 'combat.death.message', causeSource, expectedMessageKey }
}

// Vanilla death message keys for parity testing
export const VANILLA_DEATH_MESSAGES = {
  fall: 'death.attack.fall',
  fire: 'death.attack.onFire',
  drown: 'death.attack.drown',
  suffocation: 'death.attack.inWall',
  void: 'death.attack.outOfWorld',
  mob_attack: 'death.attack.mob',
  player_attack: 'death.attack.player',
  arrow: 'death.attack.arrow',
  fireball: 'death.attack.fireball',
  tnt: 'death.attack.explosion'
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('combat_damage_scenarios defined — run via the test harness')
}
