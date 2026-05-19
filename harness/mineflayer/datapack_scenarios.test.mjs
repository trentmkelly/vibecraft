import test from 'node:test'
import assert from 'node:assert/strict'
import {
  buildDatapackReloadEvidence,
  buildFeatureFlagDatapackMismatchEvidence,
  createDatapackScenarioPlan,
  summarizeDatapackEvidence
} from './datapack_scenarios.mjs'

test('createDatapackScenarioPlan covers reload and feature-flag mismatch workflows', () => {
  const plan = createDatapackScenarioPlan()

  assert.equal(plan.name, 'mineflayer-offline-datapack-scenarios')
  assert.equal(plan.mode, 'offline')
  assert.equal(plan.auth, 'offline')
  assert.deepEqual(plan.scenarios.map(scenario => scenario.name), [
    'datapack-reload',
    'feature-flag-datapack-mismatch'
  ])
})

test('datapack scenarios include reload survival, registry resync, mismatch, and disconnect evidence', () => {
  const scenarios = Object.fromEntries(createDatapackScenarioPlan().scenarios.map(scenario => [scenario.name, scenario.required]))

  assert.ok(scenarios['datapack-reload'].includes('join-before-reload'))
  assert.ok(scenarios['datapack-reload'].includes('registry-tag-resync-observed'))
  assert.ok(scenarios['datapack-reload'].includes('disconnect-reason-recorded-when-vanilla-kicks'))
  assert.ok(scenarios['feature-flag-datapack-mismatch'].includes('changed-enabled-features'))
  assert.ok(scenarios['feature-flag-datapack-mismatch'].includes('changed-datapack-registry-contents'))
  assert.ok(scenarios['feature-flag-datapack-mismatch'].includes('vanilla-compatible-success-or-disconnect'))
})

test('summarizeDatapackEvidence passes complete evidence and fails closed on missing surfaces', () => {
  const plan = createDatapackScenarioPlan()
  const complete = Object.fromEntries(plan.scenarios.map(scenario => [
    scenario.name,
    Object.fromEntries(scenario.required.map(key => [key, true]))
  ]))

  assert.equal(summarizeDatapackEvidence(complete, plan).ok, true)
  const incomplete = summarizeDatapackEvidence({
    'datapack-reload': {
      'join-before-reload': true
    }
  }, plan)
  assert.equal(incomplete.ok, false)
  assert.ok(incomplete.scenarios.find(result => result.name === 'datapack-reload').missing.includes('run-reload-command'))
  assert.ok(incomplete.scenarios.find(result => result.name === 'feature-flag-datapack-mismatch').missing.includes('offline-login-attempt'))
})

test('buildDatapackReloadEvidence captures before/after reload survival and disconnect parity', () => {
  const evidence = buildDatapackReloadEvidence({
    beforeReload: { joined: true, registryHash: 'a', tagHash: 'a' },
    afterReload: { joined: true, reloadCommandSent: true, registryHash: 'a', tagHash: 'b' },
    official: { afterReload: { joined: true } }
  })

  assert.equal(evidence['join-before-reload'], true)
  assert.equal(evidence['run-reload-command'], true)
  assert.equal(evidence['registry-tag-resync-observed'], true)
  assert.equal(evidence['bot-survives-where-vanilla-survives'], true)
  assert.equal(evidence['disconnect-reason-recorded-when-vanilla-kicks'], true)

  const kickEvidence = buildDatapackReloadEvidence({
    beforeReload: { joined: true, registryHash: 'a', tagHash: 'a' },
    afterReload: { joined: false, reloadCommandSent: true, registryHash: 'b', tagHash: 'a', disconnectReason: 'Registry reload failed' },
    official: { afterReload: { joined: false } }
  })
  assert.equal(kickEvidence['disconnect-reason-recorded-when-vanilla-kicks'], true)
})

test('buildFeatureFlagDatapackMismatchEvidence compares offline login outcome with vanilla', () => {
  const matchingKick = buildFeatureFlagDatapackMismatchEvidence({
    attempt: {
      changedEnabledFeatures: true,
      changedDatapackRegistryContents: true,
      offlineLoginAttempted: true,
      joined: false,
      disconnectReason: 'Feature flags are not compatible'
    },
    official: {
      joined: false,
      disconnectReason: 'Feature flags are not compatible'
    }
  })

  assert.equal(matchingKick['changed-enabled-features'], true)
  assert.equal(matchingKick['changed-datapack-registry-contents'], true)
  assert.equal(matchingKick['offline-login-attempt'], true)
  assert.equal(matchingKick['vanilla-compatible-success-or-disconnect'], true)
  assert.equal(matchingKick['disconnect-component-parity'], true)

  const mismatch = buildFeatureFlagDatapackMismatchEvidence({
    attempt: {
      changedEnabledFeatures: true,
      changedDatapackRegistryContents: true,
      offlineLoginAttempted: true,
      joined: true
    },
    official: {
      joined: false,
      disconnectReason: 'Feature flags are not compatible'
    }
  })
  assert.equal(mismatch['vanilla-compatible-success-or-disconnect'], false)
  assert.equal(mismatch['disconnect-component-parity'], false)
})
