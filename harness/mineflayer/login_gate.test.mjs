import test from 'node:test'
import assert from 'node:assert/strict'
import {
  changedFilesFromEnv,
  formatLoginGateReport,
  runRequiredLoginGate,
  shouldRunLoginGate
} from './login_gate.mjs'

test('shouldRunLoginGate matches network, configuration, player, storage, tick, and harness paths', () => {
  assert.equal(shouldRunLoginGate(['src/network/play.rs']), true)
  assert.equal(shouldRunLoginGate(['src/configuration.rs']), true)
  assert.equal(shouldRunLoginGate(['src/player_list.rs']), true)
  assert.equal(shouldRunLoginGate(['src/storage/chunk.rs']), true)
  assert.equal(shouldRunLoginGate(['src/tick_loop.rs']), true)
  assert.equal(shouldRunLoginGate(['harness/mineflayer/login_gate.mjs']), true)
  assert.equal(shouldRunLoginGate(['README.md', 'src/worldgen.rs']), false)
})

test('changedFilesFromEnv accepts comma and newline separated file lists', () => {
  assert.deepEqual(changedFilesFromEnv({
    VIBECRAFT_CHANGED_FILES: 'src/network/play.rs,README.md\nsrc/player.rs'
  }), ['src/network/play.rs', 'README.md', 'src/player.rs'])
})

test('runRequiredLoginGate skips when changed files do not touch gated paths', async () => {
  const result = await runRequiredLoginGate({
    changedFiles: ['README.md'],
    runLogin: async () => {
      throw new Error('should not run')
    }
  })
  assert.equal(result.ok, true)
  assert.equal(result.skipped, true)
})

test('runRequiredLoginGate runs login and packet smoke checks for gated paths', async () => {
  const result = await runRequiredLoginGate({
    changedFiles: ['src/network/play.rs'],
    runLogin: async () => completeSession()
  })
  assert.equal(result.ok, true)
  assert.equal(result.skipped, false)
  assert.match(result.report, /PASS offlineLogin/)
})

test('runRequiredLoginGate fails on login errors or smoke failures', async () => {
  const errored = completeSession()
  errored.error = new Error('login timeout')
  const errorResult = await runRequiredLoginGate({ runLogin: async () => errored })
  assert.equal(errorResult.ok, false)
  assert.match(errorResult.report, /error: login timeout/)

  const missingPackets = completeSession()
  missingPackets.packetTrace = []
  const smokeResult = await runRequiredLoginGate({ runLogin: async () => missingPackets })
  assert.equal(smokeResult.ok, false)
  assert.match(smokeResult.report, /FAIL configurationCompletion/)
})

test('formatLoginGateReport includes profile, uuid, events, and smoke details', () => {
  const session = completeSession()
  const smoke = { checks: [{ ok: true, name: 'offlineLogin', details: { checks: 3 } }] }
  assert.equal(formatLoginGateReport(session, smoke), [
    'profile: VibeCraftGate',
    'uuid: gate-uuid',
    'events: login,packet,spawn,message,action',
    'PASS offlineLogin: {"checks":3}'
  ].join('\n'))
})

function completeSession() {
  return {
    profile: { username: 'VibeCraftGate' },
    uuid: 'gate-uuid',
    timeline: [
      { name: 'login', summary: [] },
      { name: 'packet', summary: ['success'] },
      { name: 'spawn', summary: [] },
      { name: 'message', summary: ['hello'] },
      { name: 'action', summary: ['issueCommand', { command: '/list' }] }
    ],
    packetTrace: [
      { direction: 'inbound', state: 'login', name: 'success', keys: ['uuid'] },
      { direction: 'outbound', state: 'configuration', name: 'finish_configuration', keys: [] },
      { direction: 'inbound', state: 'play', name: 'keep_alive', keys: ['keepAliveId'] },
      { direction: 'outbound', state: 'play', name: 'keep_alive', keys: ['keepAliveId'] }
    ]
  }
}
