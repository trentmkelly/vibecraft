import test from 'node:test'
import assert from 'node:assert/strict'
import { EventEmitter } from 'node:events'
import {
  assertSpawnProtection,
  createBlockInteractionScenarioPlan,
  digBlock,
  expectDeniedInteraction,
  placeBlock,
  useBlock,
  waitForBlockUpdate
} from './block_interactions.mjs'

test('digBlock uses Mineflayer dig and records the target block', async () => {
  const session = fakeSession()
  const result = await digBlock(session, { x: 1, y: 64, z: 2 })
  assert.deepEqual(session.bot.calls[0], ['dig', block('stone', { x: 1, y: 64, z: 2 }), true])
  assert.equal(result.before.name, 'stone')
  assert.equal(result.result, 'dug')
})

test('placeBlock uses a reference block and face vector', async () => {
  const session = fakeSession()
  const result = await placeBlock(session, { x: 1, y: 64, z: 2 }, { faceVector: { x: 0, y: 0, z: 1 } })
  assert.deepEqual(session.bot.calls[0], [
    'placeBlock',
    block('stone', { x: 1, y: 64, z: 2 }),
    { x: 0, y: 0, z: 1 }
  ])
  assert.deepEqual(result.faceVector, { x: 0, y: 0, z: 1 })
})

test('useBlock prefers activateBlock and records use result', async () => {
  const session = fakeSession()
  const result = await useBlock(session, { x: 3, y: 64, z: 4 }, { direction: { x: 0, y: 1, z: 0 } })
  assert.deepEqual(session.bot.calls[0], [
    'activateBlock',
    block('stone', { x: 3, y: 64, z: 4 }),
    { x: 0, y: 1, z: 0 }
  ])
  assert.equal(result.result, 'used')
})

test('useBlock falls back to openBlock for container-style interactions', async () => {
  const session = fakeSession()
  delete session.bot.activateBlock
  const result = await useBlock(session, { x: 5, y: 64, z: 6 })
  assert.deepEqual(session.bot.calls[0], ['openBlock', block('stone', { x: 5, y: 64, z: 6 })])
  assert.equal(result.result, '[object Object]')
})

test('expectDeniedInteraction passes for thrown denial or unchanged block state', async () => {
  const session = fakeSession()
  const denied = await expectDeniedInteraction(session, async () => {
    throw Object.assign(new Error('cannot build here'), { code: 'spawn_protected' })
  }, {
    target: { x: 0, y: 64, z: 0 },
    reason: 'spawn protection'
  })
  assert.equal(denied.ok, true)
  assert.equal(denied.reason, 'spawn protection')

  const unchanged = await expectDeniedInteraction(session, async () => {}, {
    blockBefore: block('stone', { x: 0, y: 64, z: 0 }),
    blockAfter: block('stone', { x: 0, y: 64, z: 0 })
  })
  assert.equal(unchanged.ok, true)
})

test('assertSpawnProtection models protected radius and operator bypass', () => {
  const session = fakeSession()
  assert.equal(assertSpawnProtection(session, {
    position: { x: 4, z: 4 },
    spawn: { x: 0, z: 0 },
    protectedRadius: 16,
    opLevel: 0,
    denied: true
  }).ok, true)
  assert.equal(assertSpawnProtection(session, {
    position: { x: 4, z: 4 },
    spawn: { x: 0, z: 0 },
    protectedRadius: 16,
    opLevel: 4,
    denied: false
  }).ok, true)
})

test('waitForBlockUpdate verifies client-visible block update events', async () => {
  const session = fakeSession()
  const waiter = waitForBlockUpdate(session, { name: 'diamond_block', position: { x: 1, y: 64, z: 1 } }, { timeoutMs: 100 })
  session.bot.emit(
    'blockUpdate',
    block('stone', { x: 1, y: 64, z: 1 }),
    block('diamond_block', { x: 1, y: 64, z: 1 })
  )
  const result = await waiter
  assert.equal(result.ok, true)
  assert.equal(result.oldBlock.name, 'stone')
  assert.equal(result.newBlock.name, 'diamond_block')
})

test('createBlockInteractionScenarioPlan covers the checklist interaction surface', () => {
  const plan = createBlockInteractionScenarioPlan({ protectedRadius: 8 })
  assert.deepEqual(plan.steps.map(step => step.action), [
    'block.dig',
    'block.place',
    'block.use',
    'block.denied',
    'block.spawn_protection',
    'block.update_visible'
  ])
  assert.equal(plan.steps[4].protectedRadius, 8)
  assert.equal(plan.mode, 'offline')
})

function fakeSession() {
  const bot = new EventEmitter()
  bot.calls = []
  bot.blockAt = position => block('stone', position)
  bot.dig = async (target, forceLook) => {
    bot.calls.push(['dig', target, forceLook])
    return 'dug'
  }
  bot.placeBlock = async (target, faceVector) => {
    bot.calls.push(['placeBlock', target, faceVector])
    return 'placed'
  }
  bot.activateBlock = async (target, direction) => {
    bot.calls.push(['activateBlock', target, direction])
    return 'used'
  }
  bot.openBlock = async target => {
    bot.calls.push(['openBlock', target])
    return { id: 2, type: 'minecraft:chest' }
  }
  return { bot, timeline: [] }
}

function block(name, position) {
  return {
    name,
    type: blockType(name),
    position,
    boundingBox: 'block',
    metadata: 0
  }
}

function blockType(name) {
  return {
    diamond_block: 57,
    stone: 1
  }[name] ?? 0
}
