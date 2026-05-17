import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createVanillaCompatibilityPlan,
  summarizeCompatibilityEvidence
} from './vanilla_compatibility.mjs'

test('createVanillaCompatibilityPlan covers vanilla 26.1.2 client workflows', () => {
  const plan = createVanillaCompatibilityPlan({ version: '1.21.6' })

  assert.equal(plan.name, 'vanilla-26.1.2-compatibility')
  assert.equal(plan.targetServer, 'minecraft_server.26.1.2')
  assert.equal(plan.version, '1.21.6')
  assert.equal(plan.serverProperties.gamemode, 'survival')
  assert.equal(plan.serverProperties['allow-nether'], 'true')
  assert.deepEqual(plan.workflows.map(workflow => workflow.name), [
    'client-join',
    'survival-play',
    'death-respawn',
    'dimension-travel',
    'saving',
    'reconnecting',
    'shutdown'
  ])
})

test('summarizeCompatibilityEvidence accepts complete join, play, respawn, dimension, save, reconnect, and shutdown evidence', () => {
  const plan = createVanillaCompatibilityPlan()
  const session = {
    paths: { world: '/tmp/world' },
    timeline: [
      { name: 'login' },
      { name: 'spawn' },
      { name: 'death' },
      { name: 'end' },
      { name: 'action', summary: ['move', {}] },
      { name: 'action', summary: ['dig', {}] },
      { name: 'action', summary: ['place', {}] },
      { name: 'action', summary: ['issueCommand', { command: '/say compatibility' }] },
      { name: 'action', summary: ['dimension-change', { dimension: 'minecraft:the_nether' }] },
      { name: 'action', summary: ['save-all', {}] },
      { name: 'action', summary: ['forceReconnect.end', {}] },
      { name: 'action', summary: ['forceReconnect.connected', {}] },
      { name: 'action', summary: ['clean-shutdown-ready', {}] }
    ],
    packetTrace: [
      { name: 'login', state: 'play' },
      { name: 'finish_configuration', state: 'configuration' }
    ]
  }

  const summary = summarizeCompatibilityEvidence(session, plan)
  assert.equal(summary.ok, true)
  assert.deepEqual(summary.workflows.map(workflow => workflow.ok), [
    true,
    true,
    true,
    true,
    true,
    true,
    true
  ])
})

test('summarizeCompatibilityEvidence fails missing workflow evidence or unexpected kicks', () => {
  const plan = createVanillaCompatibilityPlan()
  const summary = summarizeCompatibilityEvidence({
    paths: {},
    timeline: [
      { name: 'login' },
      { name: 'kicked', summary: ['disconnect'] }
    ],
    packetTrace: []
  }, plan)

  assert.equal(summary.ok, false)
  assert.ok(summary.workflows.some(workflow => workflow.ok === false))
  assert.equal(summary.unexpectedKicks.length, 1)
})
