import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createMultiBotSoakPlan,
  summarizeMultiBotSoak
} from './multi_bot_soak.mjs'

test('createMultiBotSoakPlan covers repeated offline joins, leaves, movement, keepalives, chat, and reconnects', () => {
  const plan = createMultiBotSoakPlan({ botCount: 3, cycles: 2 })

  assert.equal(plan.name, 'mineflayer-multi-bot-offline-soak')
  assert.equal(plan.client, 'mineflayer')
  assert.equal(plan.mode, 'offline')
  assert.equal(plan.auth, 'offline')
  assert.equal(plan.botCount, 3)
  assert.equal(plan.cycles, 2)
  assert.equal(plan.profiles.length, 3)
  assert.equal(plan.serverProperties['online-mode'], 'false')
  assert.equal(plan.serverProperties['max-players'], '20')
  assert.deepEqual(plan.checks, [
    'repeated-offline-joins',
    'leaves',
    'movement-ticks',
    'keepalive-round-trips',
    'chat-broadcasts',
    'reconnects',
    'stable-offline-uuids',
    'no-unexpected-kicks'
  ])
  assert.notEqual(plan.profiles[0].expectedUuid, plan.profiles[1].expectedUuid)
})

test('summarizeMultiBotSoak accepts complete per-bot soak evidence', () => {
  const plan = createMultiBotSoakPlan({ botCount: 2, cycles: 1 })
  const sessions = plan.profiles.map(profile => ({
    profile: { ...profile, actualUuid: profile.expectedUuid },
    timeline: [
      { name: 'spawn' },
      { name: 'action', summary: ['multiBotMovementTick', { cycle: 0 }] },
      { name: 'action', summary: ['multiBotChat', { cycle: 0 }] },
      { name: 'action', summary: ['multiBotReconnect', { cycle: 0 }] },
      { name: 'end' }
    ],
    packetTrace: [{ name: 'keep_alive', state: 'play' }]
  }))

  assert.deepEqual(summarizeMultiBotSoak(sessions, plan), {
    ok: true,
    joined: 2,
    left: 2,
    moved: 2,
    chatted: 2,
    reconnected: 2,
    keepAlive: 2,
    kicked: [],
    uuidMismatches: []
  })
})

test('summarizeMultiBotSoak fails missing keepalive, reconnect, or stable offline uuid evidence', () => {
  const plan = createMultiBotSoakPlan({ botCount: 1, cycles: 1 })
  const incomplete = [{
    profile: { ...plan.profiles[0], actualUuid: '00000000-0000-0000-0000-000000000000' },
    timeline: [
      { name: 'spawn' },
      { name: 'action', summary: ['multiBotMovementTick', { cycle: 0 }] },
      { name: 'action', summary: ['multiBotChat', { cycle: 0 }] },
      { name: 'end' }
    ],
    packetTrace: []
  }]

  const summary = summarizeMultiBotSoak(incomplete, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.reconnected, 0)
  assert.equal(summary.keepAlive, 0)
  assert.equal(summary.uuidMismatches.length, 1)
})
