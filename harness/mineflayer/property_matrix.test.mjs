import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createOfflinePropertyMatrix,
  summarizePropertyMatrixEvidence
} from './property_matrix.mjs'

test('createOfflinePropertyMatrix defines generated offline profile and required property scenarios', () => {
  const matrix = createOfflinePropertyMatrix({ username: 'ConfigBot' })

  assert.equal(matrix.name, 'mineflayer-offline-property-matrix')
  assert.equal(matrix.mode, 'offline')
  assert.equal(matrix.auth, 'offline')
  assert.equal(matrix.profile.username, 'ConfigBot')
  assert.match(matrix.profile.expectedUuid, /^[0-9a-f-]{36}$/)
  assert.deepEqual(matrix.scenarios.map(scenario => scenario.name), [
    'default-server-properties-login',
    'offline-secure-default-world',
    'join-limits-and-status',
    'world-and-gameplay-initial-state',
    'single-property-login-bisect',
    'generated-properties',
    'property-minimization',
    'negative-configuration',
    'empty-working-directory-login',
    'property-roundtrip',
    'bind-address',
    'compression-property-login'
  ])
})

test('property matrix covers offline, secure-profile, limits, world, bisect, generated, negative, bind, and compression keys', () => {
  const scenarios = Object.fromEntries(createOfflinePropertyMatrix().scenarios.map(entry => [entry.name, entry]))

  assert.equal(scenarios['offline-secure-default-world'].properties['online-mode'], 'false')
  assert.equal(scenarios['offline-secure-default-world'].properties['enforce-secure-profile'], 'false')
  assert.ok(Object.hasOwn(scenarios['join-limits-and-status'].matrix, 'max-players'))
  assert.ok(Object.hasOwn(scenarios['join-limits-and-status'].matrix, 'network-compression-threshold'))
  assert.ok(Object.hasOwn(scenarios['world-and-gameplay-initial-state'].matrix, 'level-seed'))
  assert.ok(Object.hasOwn(scenarios['world-and-gameplay-initial-state'].matrix, 'spawn-protection'))
  assert.equal(scenarios['single-property-login-bisect'].togglesOnePropertyPerRun, true)
  assert.ok(scenarios['generated-properties'].startsWith.includes('deleted-server.properties'))
  assert.ok(scenarios['negative-configuration'].cases.includes('occupied-port'))
  assert.ok(scenarios['bind-address'].hosts.includes('rejected-address'))
  assert.ok(scenarios['compression-property-login'].thresholds.includes('-1'))
  assert.ok(scenarios['compression-property-login'].thresholds.includes('0'))
})

test('summarizePropertyMatrixEvidence passes complete scenario evidence', () => {
  const matrix = createOfflinePropertyMatrix()
  const evidence = Object.fromEntries(matrix.scenarios.map(scenario => [
    scenario.name,
    Object.fromEntries(requiredKeys(scenario).map(key => [key, true]))
  ]))

  const summary = summarizePropertyMatrixEvidence(evidence, matrix)
  assert.equal(summary.ok, true)
  assert.ok(summary.scenarios.every(result => result.missing.length === 0))
})

test('summarizePropertyMatrixEvidence fails closed when property scenario evidence is missing', () => {
  const summary = summarizePropertyMatrixEvidence({
    'default-server-properties-login': {
      'generated-profile': true
    }
  }, createOfflinePropertyMatrix())

  assert.equal(summary.ok, false)
  assert.ok(summary.scenarios.find(result => result.name === 'default-server-properties-login').missing.includes('first-join'))
  assert.ok(summary.scenarios.find(result => result.name === 'compression-property-login').missing.includes('packet-order-preserved'))
})

function requiredKeys(scenario) {
  return [
    ...(scenario.required ?? []),
    ...(scenario.togglesOnePropertyPerRun ? ['toggles-one-property-per-run'] : []),
    ...(scenario.reportsFirstChangedBehavior ? ['reports-first-changed-behavior'] : []),
    ...(scenario.removesOptionalKeysOneAtATime ? ['removes-optional-keys-one-at-a-time'] : [])
  ]
}
