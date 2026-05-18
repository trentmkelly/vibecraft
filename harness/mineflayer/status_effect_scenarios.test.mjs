import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createStatusEffectScenariosPlan,
  summarizeStatusEffectScenarios,
  planApplyEffect,
  planEffectStack,
  planEffectExpire,
  planEffectPersistence,
  ambientParticleAlpha
} from './status_effect_scenarios.mjs'

test('createStatusEffectScenariosPlan covers all 7 effect parity steps', () => {
  const plan = createStatusEffectScenariosPlan()
  assert.equal(plan.name, 'mineflayer-status-effect-scenarios')
  assert.equal(plan.steps.length, 7)
  assert.ok(plan.steps.includes('apply-effect-packet-received'))
  assert.ok(plan.steps.includes('save-restore-on-reconnect'))
})

test('summarizeStatusEffectScenarios returns ok when all steps evidenced', () => {
  const plan = createStatusEffectScenariosPlan()
  const actionMap = {
    'apply-effect-packet-received': 'effect.apply.packet',
    'tick-duration-countdown': 'effect.tick.countdown',
    'stack-amplifier-higher-replaces': 'effect.stack.amplifier',
    'expire-remove-packet-sent': 'effect.expire.remove_packet',
    'clear-via-milk-bucket': 'effect.clear.milk',
    'save-restore-on-reconnect': 'effect.persistence.reconnect',
    'client-particles-icons-amplifiers-durations': 'effect.client.display'
  }
  const timeline = plan.steps.map(step => ({ name: 'effect', summary: [actionMap[step], {}] }))
  assert.equal(summarizeStatusEffectScenarios({ timeline }, plan).ok, true)
})

test('summarizeStatusEffectScenarios fails with empty timeline', () => {
  const plan = createStatusEffectScenariosPlan()
  assert.equal(summarizeStatusEffectScenarios({ timeline: [] }, plan).ok, false)
})

test('planApplyEffect records effect id amplifier and duration', () => {
  const p = planApplyEffect('minecraft:speed', 1, 200)
  assert.equal(p.effectId, 'minecraft:speed')
  assert.equal(p.amplifier, 1)
  assert.equal(p.durationTicks, 200)
})

test('planEffectStack records both amplifiers and higher-wins flag', () => {
  const p = planEffectStack('minecraft:speed', 0, 2)
  assert.equal(p.lowAmplifier, 0)
  assert.equal(p.highAmplifier, 2)
  assert.equal(p.expectHigherWins, true)
})

test('planEffectExpire records remaining ticks for removal check', () => {
  const p = planEffectExpire('minecraft:poison', 0)
  assert.equal(p.remainingTicks, 0)
  assert.equal(p.effectId, 'minecraft:poison')
})

test('planEffectPersistence records save list and expect-restored flag', () => {
  const effects = [{ id: 'minecraft:speed', amplifier: 0, duration: 100 }]
  const p = planEffectPersistence(effects)
  assert.equal(p.expectRestored, true)
  assert.equal(p.effects.length, 1)
})

test('ambientParticleAlpha returns 38 for ambient beacon effects and 255 for normal', () => {
  assert.equal(ambientParticleAlpha(true), 38)
  assert.equal(ambientParticleAlpha(false), 255)
})
