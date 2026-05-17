import test from 'node:test'
import assert from 'node:assert/strict'
import {
  formatFailureMinimizerReport,
  minimizeLoginFailure,
  minimizedScenario
} from './failure_minimizer.mjs'

test('minimizedScenario reduces failing scenarios to one bot, one property file, and one world', () => {
  const minimized = minimizedScenario({
    name: 'multi-bot-login',
    version: '1.21.6',
    profiles: [{ username: 'BotA' }, { username: 'BotB' }],
    serverProperties: { 'online-mode': 'false', difficulty: 'hard' },
    timeoutMs: 45000,
    packetCapture: { states: ['login'], raw: false }
  })
  assert.equal(minimized.name, 'multi-bot-login-minimized')
  assert.deepEqual(minimized.profiles, [{ username: 'BotA' }])
  assert.equal(minimized.propertyFiles, 1)
  assert.equal(minimized.tempWorlds, 1)
  assert.deepEqual(minimized.serverProperties, { 'online-mode': 'false', difficulty: 'hard' })
  assert.deepEqual(minimized.packetCapture, { enabled: true, states: ['login'], raw: false })
})

test('minimizeLoginFailure reruns shard with minimized scenario inputs', async () => {
  let received
  const result = await minimizeLoginFailure({
    name: 'bad-login',
    version: '1.21.6',
    profiles: [{ username: 'BotA' }, { username: 'BotB' }],
    serverProperties: { 'online-mode': 'false' },
    packetCapture: { states: ['login', 'play'] }
  }, {
    runShard: async options => {
      received = options
      return { ok: false, report: 'still fails' }
    }
  })
  assert.equal(result.ok, false)
  assert.equal(received.username, 'BotA')
  assert.equal(received.version, '1.21.6')
  assert.deepEqual(received.properties, { 'online-mode': 'false' })
  assert.equal(received.packetCapture.enabled, true)
  assert.match(result.report, /result: fail/)
})

test('formatFailureMinimizerReport emits stable rerun summary', () => {
  assert.equal(formatFailureMinimizerReport({
    name: 'login-minimized',
    profiles: [{ username: 'Bot' }],
    propertyFiles: 1,
    tempWorlds: 1,
    packetCapture: { enabled: true }
  }, { ok: true }), [
    'scenario: login-minimized',
    'bot: Bot',
    'propertyFiles: 1',
    'tempWorlds: 1',
    'packetCapture: enabled',
    'result: pass'
  ].join('\n'))
})
