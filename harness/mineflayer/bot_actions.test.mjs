import test from 'node:test'
import assert from 'node:assert/strict'
import { EventEmitter } from 'node:events'
import { offlineUuid } from './runner.mjs'
import {
  assertHeldItem,
  breakBlock,
  captureVanillaComparisonTraces,
  forceReconnect,
  issueCommand,
  openWindow,
  placeBlock,
  waitForSpawn
} from './bot_actions.mjs'

test('waitForSpawn records already-spawned and pending-spawn action milestones', async () => {
  const spawned = fakeSession({ timeline: [{ name: 'spawn', summary: [] }] })
  assert.equal((await waitForSpawn(spawned)).alreadySpawned, true)

  const pending = fakeSession()
  const waiter = waitForSpawn(pending, { timeoutMs: 100 })
  pending.bot.emit('spawn')
  assert.equal((await waiter).alreadySpawned, false)
  assert.deepEqual(pending.timeline.map(event => event.summary?.[0]).filter(Boolean), ['waitForSpawn'])
})

test('assertHeldItem compares held item name, count, and display name', () => {
  const session = fakeSession({
    bot: { heldItem: { name: 'diamond_pickaxe', count: 1, displayName: 'Diamond Pickaxe' } }
  })
  assert.equal(assertHeldItem(session, { name: 'diamond_pickaxe', count: 1 }).ok, true)
  const mismatch = assertHeldItem(session, { name: 'stone', displayName: 'Stone' })
  assert.equal(mismatch.ok, false)
  assert.equal(mismatch.code, 'held_item.mismatch')
})

test('issueCommand normalizes slash commands and records the action', () => {
  const sent = []
  const session = fakeSession({ bot: { chat: command => sent.push(command) } })
  issueCommand(session, 'list')
  issueCommand(session, '/seed')
  assert.deepEqual(sent, ['/list', '/seed'])
  assert.deepEqual(session.timeline.map(event => event.summary[0]), ['issueCommand', 'issueCommand'])
})

test('placeBlock, breakBlock, and openWindow use Mineflayer block helpers', async () => {
  const calls = []
  const block = { name: 'stone', position: { x: 1, y: 64, z: 2 } }
  const session = fakeSession({
    bot: {
      blockAt: pos => ({ ...block, position: pos }),
      placeBlock: async (target, faceVector) => calls.push(['place', target.position, faceVector]),
      dig: async target => calls.push(['dig', target.position]),
      openBlock: async target => {
        calls.push(['open', target.position])
        return { id: 7, type: 'minecraft:chest', title: 'Chest' }
      }
    }
  })

  await placeBlock(session, block.position, { faceVector: { x: 0, y: 0, z: 1 } })
  await breakBlock(session, block.position)
  const window = await openWindow(session, block.position)

  assert.deepEqual(calls, [
    ['place', block.position, { x: 0, y: 0, z: 1 }],
    ['dig', block.position],
    ['open', block.position]
  ])
  assert.equal(window.type, 'minecraft:chest')
  assert.deepEqual(session.timeline.map(event => event.summary[0]), ['placeBlock', 'breakBlock', 'openWindow'])
})

test('forceReconnect ends the current bot and appends new login observations', async () => {
  let ended = false
  const session = fakeSession({
    endpoint: { host: '127.0.0.1', port: 25565, version: '1.21.6' },
    bot: { end: () => { ended = true } }
  })
  await forceReconnect(session, {
    connectBot: async options => {
      options.timeline.push({ name: 'login', summary: [] })
      options.packetTrace.push({ name: 'success', state: 'login', keys: ['uuid'] })
      return {
        bot: { reconnected: true },
        profile: {
          username: options.username,
          expectedUuid: offlineUuid(options.username),
          actualUuid: offlineUuid(options.username)
        }
      }
    }
  })
  assert.equal(ended, true)
  assert.equal(session.bot.reconnected, true)
  assert.equal(session.uuid, offlineUuid('VibeCraftBot'))
  assert.deepEqual(session.packetTrace.map(packet => packet.name), ['success'])
})

test('captureVanillaComparisonTraces preserves comparable event and packet traces', () => {
  const official = fakeSession({
    profile: { username: 'Bot', expectedUuid: 'a', actualUuid: 'a' },
    uuid: 'a',
    timeline: [{ name: 'spawn', at: 1, summary: [] }],
    packetTrace: [{ name: 'login', state: 'play', at: 1, keys: ['entityId'] }]
  })
  const vibeCraft = fakeSession({
    profile: { username: 'Bot', expectedUuid: 'a', actualUuid: 'a' },
    uuid: 'a',
    timeline: [{ name: 'spawn', at: 2, summary: [] }],
    packetTrace: [{ name: 'login', state: 'play', at: 2, keys: ['entityId'] }]
  })
  assert.deepEqual(captureVanillaComparisonTraces(official, vibeCraft), {
    official: {
      profile: official.profile,
      uuid: 'a',
      timeline: [{ name: 'spawn', summary: [] }],
      packetTrace: [{ name: 'login', state: 'play', keys: ['entityId'] }]
    },
    vibeCraft: {
      profile: vibeCraft.profile,
      uuid: 'a',
      timeline: [{ name: 'spawn', summary: [] }],
      packetTrace: [{ name: 'login', state: 'play', keys: ['entityId'] }]
    }
  })
})

function fakeSession(options = {}) {
  const bot = options.bot ?? new EventEmitter()
  return {
    endpoint: options.endpoint ?? { host: '127.0.0.1', port: 25565 },
    bot,
    profile: options.profile ?? {
      username: 'VibeCraftBot',
      expectedUuid: offlineUuid('VibeCraftBot'),
      actualUuid: offlineUuid('VibeCraftBot')
    },
    uuid: options.uuid ?? offlineUuid('VibeCraftBot'),
    timeline: options.timeline ?? [],
    packetTrace: options.packetTrace ?? []
  }
}
