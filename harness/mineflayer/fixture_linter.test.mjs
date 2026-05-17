import test from 'node:test'
import assert from 'node:assert/strict'
import {
  assertMineflayerFixture,
  formatFixtureLint,
  lintMineflayerFixture
} from './fixture_linter.mjs'

test('lintMineflayerFixture accepts complete offline-mode scenario metadata', () => {
  assert.equal(lintMineflayerFixture(validFixture()).ok, true)
})

test('lintMineflayerFixture rejects missing required fixture fields', () => {
  const result = lintMineflayerFixture({})
  assert.equal(result.ok, false)
  assert.deepEqual(result.issues.map(issue => issue.path), [
    'version',
    'serverProperties',
    'profiles',
    'timeoutMs',
    'packetCapture',
    'vanillaComparison'
  ])
})

test('lintMineflayerFixture enforces offline mode and expected UUIDs', () => {
  const fixture = validFixture()
  fixture.serverProperties['online-mode'] = 'true'
  fixture.profiles = [{ username: 'Bot' }]
  const result = lintMineflayerFixture(fixture)
  assert.deepEqual(result.issues.map(issue => issue.path), [
    'serverProperties.online-mode',
    'profiles.0.expectedUuid'
  ])
})

test('lintMineflayerFixture enforces packet capture states and comparison mode', () => {
  const fixture = validFixture()
  fixture.packetCapture = { enabled: false, states: ['login'] }
  fixture.vanillaComparison = { mode: 'sometimes' }
  const result = lintMineflayerFixture(fixture)
  assert.deepEqual(result.issues.map(issue => issue.path), [
    'packetCapture.enabled',
    'packetCapture.states',
    'packetCapture.states',
    'vanillaComparison.mode'
  ])
})

test('assertMineflayerFixture returns valid fixtures and throws formatted lint failures', () => {
  const fixture = validFixture()
  assert.equal(assertMineflayerFixture(fixture), fixture)
  assert.throws(() => assertMineflayerFixture({}), /version: scenario must pin/)
})

test('formatFixtureLint emits stable lines', () => {
  assert.equal(formatFixtureLint({ ok: true, issues: [] }), 'fixture lint passed')
  assert.equal(formatFixtureLint({
    ok: false,
    issues: [{ path: 'timeoutMs', message: 'bad timeout' }]
  }), 'timeoutMs: bad timeout')
})

function validFixture() {
  return {
    version: '1.21.6',
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    },
    profiles: [{
      username: 'RustCraftBot',
      expectedUuid: '1f8d22b6-2bde-3ef7-8a05-0806e106d497'
    }],
    timeoutMs: 30000,
    packetCapture: {
      enabled: true,
      states: ['login', 'configuration', 'play']
    },
    vanillaComparison: {
      mode: 'required'
    }
  }
}
