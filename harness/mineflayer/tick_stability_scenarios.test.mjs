import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createTickStabilityPlan,
  summarizeTickStabilityEvidence
} from './tick_stability_scenarios.mjs'

test('createTickStabilityPlan covers pause, autosave, save commands, keepalives, and visible state', () => {
  const plan = createTickStabilityPlan()

  assert.equal(plan.name, 'mineflayer-offline-tick-stability')
  assert.deepEqual(plan.transitions, [
    'pause-when-empty',
    'autosave',
    'save-off',
    'save-on',
    'save-all'
  ])
  assert.deepEqual(plan.commands, ['/save-off', '/save-on', '/save-all'])
  assert.ok(plan.requiredEvidence.includes('keepalive-request-response-after-each-transition'))
  assert.ok(plan.requiredEvidence.includes('visible-state-does-not-stall'))
})

test('summarizeTickStabilityEvidence passes complete evidence', () => {
  const plan = createTickStabilityPlan()
  const evidence = {
    transitions: Object.fromEntries(plan.transitions.map(transition => [transition, true])),
    ...Object.fromEntries(plan.requiredEvidence.map(key => [key, true]))
  }

  assert.deepEqual(summarizeTickStabilityEvidence(evidence, plan), {
    ok: true,
    missing: [],
    transitions: {
      'pause-when-empty': true,
      autosave: true,
      'save-off': true,
      'save-on': true,
      'save-all': true
    },
    missingTransitions: []
  })
})

test('summarizeTickStabilityEvidence fails closed for missing keepalive or transition evidence', () => {
  const summary = summarizeTickStabilityEvidence({
    transitions: {
      autosave: true
    },
    'visible-state-does-not-stall': true
  }, createTickStabilityPlan())

  assert.equal(summary.ok, false)
  assert.ok(summary.missing.includes('keepalive-request-response-after-each-transition'))
  assert.ok(summary.missingTransitions.includes('pause-when-empty'))
  assert.ok(summary.missingTransitions.includes('save-all'))
})
