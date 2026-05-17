import test from 'node:test'
import assert from 'node:assert/strict'
import { EventEmitter } from 'node:events'
import {
  attackEntity,
  createEntityObservationScenarioPlan,
  waitForDamageAnimation,
  waitForEntityDespawn,
  waitForEntitySpawn,
  waitForItemPickup,
  waitForMetadataUpdate
} from './entity_observations.mjs'

test('waitForEntitySpawn records matching Mineflayer entity spawns', async () => {
  const session = fakeSession()
  const waiter = waitForEntitySpawn(session, { name: 'zombie', id: 7 }, { timeoutMs: 100 })
  session.bot.emit('entitySpawn', entity('zombie', 7))
  const result = await waiter
  assert.equal(result.ok, true)
  assert.equal(result.observed.name, 'zombie')
})

test('waitForEntityDespawn records matching entityGone events', async () => {
  const session = fakeSession()
  const waiter = waitForEntityDespawn(session, { name: 'zombie' }, { timeoutMs: 100 })
  session.bot.emit('entityGone', entity('zombie', 7))
  assert.equal((await waiter).ok, true)
})

test('waitForMetadataUpdate compares entity and metadata payloads', async () => {
  const session = fakeSession()
  const waiter = waitForMetadataUpdate(session, {
    entity: { name: 'zombie' },
    metadata: { pose: 'crouching', flags: 2 }
  }, { timeoutMs: 100 })
  session.bot.emit('entityUpdate', entity('zombie', 7), { pose: 'crouching', flags: 2 })
  const result = await waiter
  assert.equal(result.ok, true)
  assert.equal(result.observed.metadata.pose, 'crouching')
})

test('waitForDamageAnimation records entityHurt observations', async () => {
  const session = fakeSession()
  const waiter = waitForDamageAnimation(session, { name: 'zombie', health: 14 }, { timeoutMs: 100 })
  session.bot.emit('entityHurt', entity('zombie', 7, { health: 14 }))
  assert.equal((await waiter).ok, true)
})

test('waitForItemPickup records collector and item entity pair', async () => {
  const session = fakeSession()
  const waiter = waitForItemPickup(session, {
    collector: { name: 'RustCraftBot' },
    collected: { name: 'item' }
  }, { timeoutMs: 100 })
  session.bot.emit('playerCollect', entity('RustCraftBot', 1), entity('item', 9, { type: 'object' }))
  const result = await waiter
  assert.equal(result.ok, true)
  assert.equal(result.observed.collected.name, 'item')
})

test('attackEntity resolves a target and calls Mineflayer attack', async () => {
  const session = fakeSession()
  const result = await attackEntity(session, 'zombie', { swing: false })
  assert.deepEqual(session.bot.calls, [['attack', entity('zombie', 7), false]])
  assert.equal(result.target.name, 'zombie')
  assert.equal(result.swing, false)
})

test('createEntityObservationScenarioPlan covers the checklist observation surface', () => {
  const plan = createEntityObservationScenarioPlan()
  assert.deepEqual(plan.steps.map(step => step.action), [
    'entity.spawn',
    'entity.metadata',
    'entity.damage_animation',
    'entity.item_pickup',
    'entity.combat',
    'entity.despawn'
  ])
  assert.equal(plan.mode, 'offline')
})

function fakeSession() {
  const bot = new EventEmitter()
  bot.entities = { 7: entity('zombie', 7) }
  bot.calls = []
  bot.attack = async (target, swing) => {
    bot.calls.push(['attack', target, swing])
    return 'attacked'
  }
  return { bot, timeline: [] }
}

function entity(name, id, overrides = {}) {
  return {
    id,
    name,
    type: 'mob',
    metadata: {},
    position: { x: id, y: 64, z: -id },
    ...overrides
  }
}
