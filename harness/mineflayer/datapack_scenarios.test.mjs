import test from 'node:test'
import assert from 'node:assert/strict'
import {
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
