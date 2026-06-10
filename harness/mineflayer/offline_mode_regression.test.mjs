import test from 'node:test'
import assert from 'node:assert/strict'
import {
  evaluateOfflineModeRegression,
  formatOfflineModeRegressionReport,
  shouldRunOfflineModeRegression
} from './offline_mode_regression.mjs'

test('shouldRunOfflineModeRegression gates protocol and login path changes', () => {
  assert.equal(shouldRunOfflineModeRegression(['src/network/login.rs']), true)
  assert.equal(shouldRunOfflineModeRegression(['src/registry/mod.rs']), true)
  assert.equal(shouldRunOfflineModeRegression(['harness/mineflayer/offline_mode_regression.mjs']), true)
  assert.equal(shouldRunOfflineModeRegression(['docs/readme.md']), false)
})

test('evaluateOfflineModeRegression passes complete offline-mode login evidence', () => {
  const session = completeSession()
  const result = evaluateOfflineModeRegression(session, {
    expectedRegistryOrder: ['minecraft:dimension_type', 'minecraft:worldgen/biome'],
    minimumPlayPackets: 2
  })

  assert.equal(result.ok, true)
  assert.ok(result.checks.some(check => check.name === 'loginTimeout' && check.ok))
  assert.ok(result.checks.some(check => check.name === 'unexpectedKick' && check.ok))
  assert.ok(result.checks.some(check => check.name === 'registryOrder' && check.ok))
  assert.ok(result.checks.some(check => check.name === 'playStateStall' && check.ok))
})

test('evaluateOfflineModeRegression fails login timeout, unexpected kick, registry drift, and play-state stall', () => {
  const session = {
    ...completeSession(),
    error: new Error('Timed out waiting for spawn'),
    timeline: [
      { name: 'login' },
      { name: 'kicked', summary: ['disconnect'] }
    ],
    packetTrace: [
      { direction: 'inbound', state: 'configuration', name: 'registry_data', registryId: 'minecraft:wrong' }
    ]
  }
  const result = evaluateOfflineModeRegression(session, {
    expectedRegistryOrder: ['minecraft:dimension_type'],
    minimumPlayPackets: 1
  })

  assert.equal(result.ok, false)
  assert.ok(result.checks.find(check => check.name === 'loginTimeout').message.includes('timed out'))
  assert.ok(result.checks.find(check => check.name === 'unexpectedKick').message.includes('kicked'))
  assert.ok(result.checks.find(check => check.name === 'registryOrder').message.includes('drift'))
  assert.ok(result.checks.find(check => check.name === 'playStateStall').message.includes('stalled'))
})

test('formatOfflineModeRegressionReport emits stable PASS and FAIL lines', () => {
  const session = completeSession()
  const result = evaluateOfflineModeRegression(session, { minimumPlayPackets: 2 })
  const report = formatOfflineModeRegressionReport(session, result)

  assert.match(report, /profile: VibeCraftOfflineRegression/)
  assert.match(report, /PASS loginTimeout/)
  assert.match(report, /PASS playStateStall/)
})

function completeSession() {
  return {
    profile: { username: 'VibeCraftOfflineRegression' },
    uuid: 'offline-uuid',
    timeline: [
      { name: 'login' },
      { name: 'packet', summary: ['success'] },
      { name: 'spawn' },
      { name: 'action', summary: ['issueCommand', { command: '/list' }] }
    ],
    packetTrace: [
      { direction: 'inbound', state: 'login', name: 'success', keys: ['uuid', 'username'] },
      {
        direction: 'inbound',
        state: 'configuration',
        name: 'registry_data',
        registryId: 'minecraft:dimension_type',
        keys: ['registryCodec']
      },
      {
        direction: 'inbound',
        state: 'configuration',
        name: 'registry_data',
        registryId: 'minecraft:worldgen/biome',
        keys: ['registryCodec']
      },
      { direction: 'inbound', state: 'configuration', name: 'finish_configuration', keys: [] },
      { direction: 'inbound', state: 'play', name: 'login', keys: ['entityId'] },
      { direction: 'inbound', state: 'play', name: 'keep_alive', keys: ['keepAliveId'] },
      { direction: 'outbound', state: 'play', name: 'keep_alive', keys: ['keepAliveId'] },
      { direction: 'inbound', state: 'play', name: 'system_chat', keys: ['content'] }
    ]
  }
}
