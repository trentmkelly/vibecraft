import test from 'node:test'
import assert from 'node:assert/strict'
import {
  ENTITY_BEHAVIOR_JAVA_ORACLES,
  ENTITY_BEHAVIOR_STEPS,
  createEntityBehaviorParityPlan,
  recordEntityBehaviorEvidence,
  requiredEvidenceForStep,
  runEntityBehaviorParityScenario,
  summarizeEntityBehaviorParity
} from './entity_behavior_parity.mjs'

test('createEntityBehaviorParityPlan covers every broad entity behavior surface', () => {
  const plan = createEntityBehaviorParityPlan({ username: 'Verifier' })

  assert.equal(plan.name, 'mineflayer-entity-behavior-parity')
  assert.equal(plan.username, 'Verifier')
  assert.equal(plan.comparedAgainst, 'official-server.jar')
  assert.deepEqual(plan.steps.map(step => step.id), [
    'spawning-finalize-spawn',
    'ai-goal-tick',
    'pathfinding-navigation-tick',
    'combat-damage-animation',
    'drops-loot-table',
    'save-load-roundtrip',
    'network-metadata-sync'
  ])
  for (const step of plan.steps) {
    assert.equal(step.oracle, ENTITY_BEHAVIOR_JAVA_ORACLES[step.id])
    assert.deepEqual(step.requiredEvidence, requiredEvidenceForStep(step.id))
    assert.ok(step.requiredEvidence.length > 0)
  }
})

test('summarizeEntityBehaviorParity fails closed without vanilla comparisons', () => {
  const evidence = { timeline: [] }
  for (const step of ENTITY_BEHAVIOR_STEPS) {
    recordEntityBehaviorEvidence(evidence, step, completeDetails(step, { comparedToVanilla: false }))
  }

  const summary = summarizeEntityBehaviorParity(evidence)
  assert.equal(summary.ok, false)
  assert.ok(Object.values(summary.steps).every(value => value === false))
})

test('summarizeEntityBehaviorParity requires all evidence keys per surface', () => {
  const evidence = { comparedAgainst: 'official-server.jar', timeline: [] }
  for (const step of ENTITY_BEHAVIOR_STEPS) {
    const details = completeDetails(step)
    delete details[requiredEvidenceForStep(step)[0]]
    recordEntityBehaviorEvidence(evidence, step, details)
  }

  const summary = summarizeEntityBehaviorParity(evidence)
  assert.equal(summary.ok, false)
  assert.ok(Object.values(summary.steps).every(value => value === false))
})

test('runEntityBehaviorParityScenario accepts complete official-vs-VibeCraft evidence', async () => {
  const result = await runEntityBehaviorParityScenario({
    probe: async plan => {
      const evidence = { comparedAgainst: 'official-server.jar', timeline: [] }
      for (const step of plan.steps) {
        recordEntityBehaviorEvidence(evidence, step.id, completeDetails(step.id))
      }
      return evidence
    }
  })

  assert.equal(result.summary.ok, true)
  assert.equal(result.summary.comparedAgainst, 'official-server.jar')
  assert.deepEqual(Object.keys(result.summary.steps), ENTITY_BEHAVIOR_STEPS)
})

test('runEntityBehaviorParityScenario reports missing live probe by default', async () => {
  await assert.rejects(
    () => runEntityBehaviorParityScenario(),
    /requires live VibeCraft and official-server\.jar fixtures/
  )
})

function completeDetails(step, overrides = {}) {
  return {
    comparedToVanilla: true,
    ...Object.fromEntries(requiredEvidenceForStep(step).map(key => [key, `${step}:${key}`])),
    ...overrides
  }
}
