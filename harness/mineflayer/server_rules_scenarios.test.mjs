import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createServerRulesScenarioPlan,
  summarizeServerRulesEvidence
} from './server_rules_scenarios.mjs'

test('createServerRulesScenarioPlan covers max-player, whitelist, mutable, reload, and secure-profile rules', () => {
  const plan = createServerRulesScenarioPlan()

  assert.equal(plan.name, 'mineflayer-offline-server-rules')
  assert.equal(plan.mode, 'offline')
  assert.equal(plan.auth, 'offline')
  assert.deepEqual(plan.scenarios.map(scenario => scenario.name), [
    'max-player-enforcement',
    'whitelist',
    'mutable-properties',
    'configuration-reload',
    'secure-profile-toggle'
  ])
})

test('server rule scenarios include each checklist-required behavior', () => {
  const scenarios = Object.fromEntries(createServerRulesScenarioPlan().scenarios.map(entry => [entry.name, entry.steps]))

  assert.ok(scenarios['max-player-enforcement'].includes('vanilla-full-server-disconnect'))
  assert.ok(scenarios.whitelist.includes('offline-allow'))
  assert.ok(scenarios.whitelist.includes('offline-deny'))
  assert.ok(scenarios.whitelist.includes('runtime-whitelist-reload'))
  assert.ok(scenarios.whitelist.includes('enforce-whitelist-toggle'))
  assert.ok(scenarios['mutable-properties'].includes('motd-change'))
  assert.ok(scenarios['mutable-properties'].includes('view-distance-change'))
  assert.ok(scenarios['mutable-properties'].includes('reconnecting-bot-observes-state'))
  assert.ok(scenarios['configuration-reload'].includes('restart-required-properties-unchanged'))
  assert.ok(scenarios['configuration-reload'].includes('reloadable-properties-applied'))
  assert.ok(scenarios['secure-profile-toggle'].includes('enforce-secure-profile-false-allows-generated-offline-bot'))
  assert.ok(scenarios['secure-profile-toggle'].includes('unsigned-mineflayer-client-behavior'))
})

test('summarizeServerRulesEvidence passes complete evidence', () => {
  const plan = createServerRulesScenarioPlan()
  const evidence = Object.fromEntries(plan.scenarios.map(entry => [
    entry.name,
    Object.fromEntries(entry.steps.map(step => [step, true]))
  ]))

  const summary = summarizeServerRulesEvidence(evidence, plan)
  assert.equal(summary.ok, true)
  assert.ok(summary.scenarios.every(result => result.missing.length === 0))
})

test('summarizeServerRulesEvidence fails closed when rule evidence is missing', () => {
  const summary = summarizeServerRulesEvidence({
    whitelist: {
      'offline-allow': true
    }
  }, createServerRulesScenarioPlan())

  assert.equal(summary.ok, false)
  assert.ok(summary.scenarios.find(result => result.name === 'whitelist').missing.includes('offline-deny'))
  assert.ok(summary.scenarios.find(result => result.name === 'max-player-enforcement').missing.includes('extra-bot-attempt'))
})
