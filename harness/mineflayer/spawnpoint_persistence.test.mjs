import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createSpawnpointPersistencePlan,
  summarizeSpawnpointPersistence,
  planBedSpawnSet,
  planAnchorSpawnSet,
  planDeathRespawn,
  planMissingSpawnFallback
} from './spawnpoint_persistence.mjs'

test('createSpawnpointPersistencePlan covers all 6 spawnpoint steps', () => {
  const plan = createSpawnpointPersistencePlan()
  assert.equal(plan.name, 'mineflayer-spawnpoint-persistence')
  assert.deepEqual(plan.compareAgainst, ['vanilla-26.1.2', 'vibecraft'])
  assert.equal(plan.steps.length, 6)
  assert.deepEqual(plan.assertions, [
    'bed-spawn-saved-to-playerdata',
    'anchor-spawn-saved-to-playerdata',
    'spawnpoint-restored-before-first-visible-spawn-after-reconnect',
    'death-respawn-uses-valid-bed-spawn',
    'death-respawn-uses-charged-anchor-spawn',
    'missing-or-invalid-spawn-falls-back-to-world-spawn'
  ])
  assert.ok(plan.steps.includes('set-bed-spawnpoint'))
  assert.ok(plan.steps.includes('reconnect-spawnpoint-restored'))
  assert.ok(plan.steps.includes('missing-spawn-fallback-world-spawn'))
})

test('summarizeSpawnpointPersistence returns ok when all steps evidenced', () => {
  const plan = createSpawnpointPersistencePlan()
  const actionMap = {
    'set-bed-spawnpoint': 'spawn.bed.set',
    'set-respawn-anchor-spawnpoint': 'spawn.anchor.set',
    'reconnect-spawnpoint-restored': 'spawn.reconnect.restored',
    'die-respawn-at-bed': 'spawn.die.bed_respawn',
    'die-respawn-at-anchor': 'spawn.die.anchor_respawn',
    'missing-spawn-fallback-world-spawn': 'spawn.fallback.world_spawn'
  }
  const timeline = plan.steps.map(step => ({
    name: 'spawn', summary: [actionMap[step], {}]
  }))
  assert.equal(summarizeSpawnpointPersistence({ timeline }, plan).ok, true)
})

test('summarizeSpawnpointPersistence fails with empty timeline', () => {
  const plan = createSpawnpointPersistencePlan()
  assert.equal(summarizeSpawnpointPersistence({ timeline: [] }, plan).ok, false)
})

test('planBedSpawnSet records position and dimension', () => {
  const p = planBedSpawnSet('SpawnBot', { x: 10, y: 64, z: 20 }, 'minecraft:overworld')
  assert.equal(p.action, 'spawn.bed.set')
  assert.equal(p.username, 'SpawnBot')
  assert.equal(p.dimension, 'minecraft:overworld')
  assert.deepEqual(p.expectedSavedFields, {
    spawn_dimension: 'minecraft:overworld',
    spawn_x: 10,
    spawn_y: 64,
    spawn_z: 20,
    spawn_forced: false
  })
})

test('planAnchorSpawnSet records nether anchor position', () => {
  const p = planAnchorSpawnSet('SpawnBot', { x: 5, y: 64, z: 5 }, 'minecraft:the_nether', 3)
  assert.equal(p.action, 'spawn.anchor.set')
  assert.equal(p.dimension, 'minecraft:the_nether')
  assert.equal(p.charges, 3)
  assert.deepEqual(p.expectedSavedFields, {
    spawn_dimension: 'minecraft:the_nether',
    spawn_x: 5,
    spawn_y: 64,
    spawn_z: 5,
    spawn_forced: false
  })
})

test('planDeathRespawn distinguishes bed vs anchor type', () => {
  const bedRespawn = planDeathRespawn('bed', { x: 10, y: 64, z: 20 })
  assert.equal(bedRespawn.action, 'spawn.die.bed_respawn')
  assert.equal(bedRespawn.spawnType, 'bed')
  assert.ok(bedRespawn.assertions.includes('client-respawn-packet-observed'))
  assert.ok(bedRespawn.assertions.includes('post-respawn-position-matches-spawnpoint'))

  const anchorRespawn = planDeathRespawn('anchor', { x: 5, y: 64, z: 5 })
  assert.equal(anchorRespawn.action, 'spawn.die.anchor_respawn')
  assert.equal(anchorRespawn.spawnType, 'anchor')
})

test('planMissingSpawnFallback records fallback reason', () => {
  const p = planMissingSpawnFallback('bed_missing')
  assert.equal(p.action, 'spawn.fallback.world_spawn')
  assert.equal(p.reason, 'bed_missing')
  assert.ok(p.assertions.includes('world-spawn-position-used'))
  assert.ok(p.assertions.includes('missing-spawn-message-or-packet-matches-vanilla'))
})
