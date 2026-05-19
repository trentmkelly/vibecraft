import test from 'node:test'
import assert from 'node:assert/strict'
import {
  DEATH_RESPAWN_SCENARIOS,
  createDeathRespawnPlan,
  summarizeDeathRespawn,
  recordDeathRespawn,
  runDeathRespawnScenario
} from './death_respawn_scenarios.mjs'

test('DEATH_RESPAWN_SCENARIOS covers both scenario kinds', () => {
  assert.ok('deathRespawnFlow' in DEATH_RESPAWN_SCENARIOS)
  assert.ok('respawnAfterRelogin' in DEATH_RESPAWN_SCENARIOS)
})

test('createDeathRespawnPlan covers all deathRespawnFlow steps', () => {
  const plan = createDeathRespawnPlan('deathRespawnFlow')
  assert.equal(plan.name, 'mineflayer-death-respawn-death-respawn-flow')
  assert.ok(plan.steps.includes('death-message-received'))
  assert.ok(plan.steps.includes('respawn-packet-received'))
  assert.ok(plan.steps.includes('post-respawn-health-full'))
  assert.ok(plan.steps.includes('keep-inventory-off-drops-items'))
  assert.ok(plan.steps.includes('keep-inventory-on-retains-items'))
  assert.ok(plan.steps.includes('keep-inventory-off-drops-xp'))
  assert.ok(plan.steps.includes('keep-inventory-on-retains-xp'))
  assert.ok(plan.steps.includes('post-respawn-ability-flags-match-game-mode'))
  assert.equal(plan.username, 'DeathRespawnBot')
})

test('createDeathRespawnPlan covers all respawnAfterRelogin steps', () => {
  const plan = createDeathRespawnPlan('respawnAfterRelogin')
  assert.ok(plan.steps.includes('killed-while-in-play'))
  assert.ok(plan.steps.includes('disconnect-on-death-screen'))
  assert.ok(plan.steps.includes('reconnect-reaches-play'))
  assert.ok(plan.steps.includes('no-extra-death-on-reconnect'))
  assert.ok(plan.steps.includes('respawn-position-matches-saved-spawn'))
  assert.equal(plan.username, 'RespawnReloginBot')
})

test('createDeathRespawnPlan throws for unknown kind', () => {
  assert.throws(() => createDeathRespawnPlan('unknownKind'), /Unknown death\/respawn scenario/)
})

test('summarizeDeathRespawn returns ok when all steps evidenced', () => {
  const plan = createDeathRespawnPlan('deathRespawnFlow')
  const evidence = { timeline: [] }
  for (const step of plan.steps) {
    recordDeathRespawn(evidence, step, {})
  }
  assert.equal(summarizeDeathRespawn(evidence, plan).ok, true)
})

test('summarizeDeathRespawn fails with empty timeline', () => {
  const plan = createDeathRespawnPlan('deathRespawnFlow')
  assert.equal(summarizeDeathRespawn({ timeline: [] }, plan).ok, false)
})

test('summarizeDeathRespawn fails when one step is missing', () => {
  const plan = createDeathRespawnPlan('respawnAfterRelogin')
  const evidence = { timeline: [] }
  // Record all but the last step
  for (const step of plan.steps.slice(0, -1)) {
    recordDeathRespawn(evidence, step, {})
  }
  assert.equal(summarizeDeathRespawn(evidence, plan).ok, false)
})

test('runDeathRespawnScenario rejects when probe throws', async () => {
  const plan = createDeathRespawnPlan('deathRespawnFlow')
  await assert.rejects(
    () => runDeathRespawnScenario('deathRespawnFlow', {
      probe: async () => { throw new Error('deathRespawnProbe requires a scenario-specific server fixture') }
    }),
    /scenario-specific server fixture/
  )
})
