import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createLongRunningSoakPlan,
  summarizeSoakHealth
} from './real_client_soak.mjs'

test('createLongRunningSoakPlan covers long-running real client soak requirements', () => {
  const plan = createLongRunningSoakPlan({ minutes: 2, reconnectEvery: 3, movementEvery: 1 })

  assert.equal(plan.name, 'real-client-long-running-soak')
  assert.equal(plan.client, 'mineflayer')
  assert.equal(plan.mode, 'offline')
  assert.equal(plan.auth, 'offline')
  assert.equal(plan.minutes, 2)
  assert.equal(plan.minimumTicks, 2 * 60 * 20)
  assert.equal(plan.reconnectEvery, 3)
  assert.equal(plan.movementEvery, 1)
  assert.equal(plan.serverProperties['online-mode'], 'false')
  assert.ok(plan.checks.includes('spawn-before-soak'))
  assert.ok(plan.checks.includes('play-packets-continue'))
  assert.ok(plan.checks.includes('keepalive-round-trips'))
  assert.ok(plan.checks.includes('movement-ticks-accepted'))
  assert.ok(plan.checks.includes('chat-command-round-trip'))
  assert.ok(plan.checks.includes('reconnect-preserves-play-state'))
  assert.ok(plan.checks.includes('no-unexpected-kick'))
  assert.ok(plan.checks.includes('clean-shutdown'))
})

test('summarizeSoakHealth accepts sessions with spawn, play packets, keepalives, movement, chat, and reconnect', () => {
  const plan = createLongRunningSoakPlan({ minutes: 1 })
  const session = {
    timeline: [
      { name: 'spawn' },
      { name: 'action', summary: ['soakMovementTick', { iteration: 1 }] },
      { name: 'action', summary: ['issueCommand', { command: '/list' }] },
      { name: 'action', summary: ['forceReconnect.connected', { username: 'VibeCraftSoak' }] }
    ],
    packetTrace: [
      { name: 'login', state: 'play' },
      { name: 'position', state: 'play' },
      { name: 'keep_alive', state: 'play' }
    ]
  }

  assert.deepEqual(summarizeSoakHealth(session, plan, { minimumPlayPackets: 2 }), {
    ok: true,
    playPackets: 3,
    keepAliveInbound: 1,
    movementTicks: 1,
    chatCommands: 1,
    reconnects: 1,
    unexpectedKicks: []
  })
})

test('summarizeSoakHealth fails on unexpected kick or missing keepalive', () => {
  const plan = createLongRunningSoakPlan({ minutes: 1 })
  const kicked = summarizeSoakHealth({
    timeline: [
      { name: 'spawn' },
      { name: 'kicked', summary: ['disconnect'] },
      { name: 'action', summary: ['soakMovementTick', {}] },
      { name: 'action', summary: ['issueCommand', {}] },
      { name: 'action', summary: ['forceReconnect.connected', {}] }
    ],
    packetTrace: [
      { name: 'keep_alive', state: 'play' },
      { name: 'position', state: 'play' }
    ]
  }, plan, { minimumPlayPackets: 1 })

  assert.equal(kicked.ok, false)

  const noKeepalive = summarizeSoakHealth({
    timeline: [
      { name: 'spawn' },
      { name: 'action', summary: ['soakMovementTick', {}] },
      { name: 'action', summary: ['issueCommand', {}] },
      { name: 'action', summary: ['forceReconnect.connected', {}] }
    ],
    packetTrace: [{ name: 'position', state: 'play' }]
  }, plan, { minimumPlayPackets: 1 })

  assert.equal(noKeepalive.ok, false)
})
