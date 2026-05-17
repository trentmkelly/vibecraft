import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createLifecycleScenarioPlan,
  summarizeLifecycleEvidence
} from './lifecycle_scenarios.mjs'

test('createLifecycleScenarioPlan covers offline-mode lifecycle checklist scenarios', () => {
  const plan = createLifecycleScenarioPlan()

  assert.equal(plan.name, 'mineflayer-offline-lifecycle')
  assert.equal(plan.mode, 'offline')
  assert.equal(plan.auth, 'offline')
  assert.equal(plan.serverProperties['online-mode'], 'false')
  assert.deepEqual(plan.scenarios.map(scenario => scenario.name), [
    'shutdown-restart',
    'crash-after-login-recovery',
    'startup-race',
    'first-login-bootstrap',
    'failed-start-cleanup',
    'lifecycle-artifacts',
    'interrupted-bootstrap',
    'eula-refusal'
  ])
})

test('lifecycle scenarios include required shutdown, crash, startup, bootstrap, cleanup, artifact, interruption, and EULA evidence', () => {
  const scenarios = Object.fromEntries(createLifecycleScenarioPlan().scenarios.map(entry => [entry.name, entry.steps]))

  assert.ok(scenarios['shutdown-restart'].includes('persisted-position'))
  assert.ok(scenarios['shutdown-restart'].includes('persisted-inventory'))
  assert.ok(scenarios['shutdown-restart'].includes('persisted-stats'))
  assert.ok(scenarios['crash-after-login-recovery'].includes('terminate-process'))
  assert.ok(scenarios['startup-race'].includes('official-timing-envelope'))
  assert.ok(scenarios['first-login-bootstrap'].includes('vanilla-order-before-play'))
  assert.ok(scenarios['failed-start-cleanup'].includes('no-partial-player-files'))
  assert.ok(scenarios['lifecycle-artifacts'].includes('post-exit-file-flush'))
  assert.ok(scenarios['interrupted-bootstrap'].includes('no-stale-world-lock'))
  assert.ok(scenarios['eula-refusal'].includes('same-bot-can-join'))
})

test('summarizeLifecycleEvidence passes complete lifecycle evidence', () => {
  const plan = createLifecycleScenarioPlan()
  const evidence = Object.fromEntries(plan.scenarios.map(entry => [
    entry.name,
    Object.fromEntries(entry.steps.map(step => [step, true]))
  ]))

  const summary = summarizeLifecycleEvidence(evidence, plan)
  assert.equal(summary.ok, true)
  assert.deepEqual(summary.scenarios.map(result => result.missing), [
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    []
  ])
})

test('summarizeLifecycleEvidence fails closed when required lifecycle evidence is missing', () => {
  const plan = createLifecycleScenarioPlan()
  const summary = summarizeLifecycleEvidence({
    'shutdown-restart': {
      join: true,
      reconnect: true
    }
  }, plan)

  assert.equal(summary.ok, false)
  assert.ok(summary.scenarios.find(result => result.name === 'shutdown-restart').missing.includes('clean-stop'))
  assert.ok(summary.scenarios.find(result => result.name === 'eula-refusal').missing.includes('login-before-eula'))
})
