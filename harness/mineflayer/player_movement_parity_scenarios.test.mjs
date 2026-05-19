import test from 'node:test'
import assert from 'node:assert/strict'
import {
  PLAYER_MOVEMENT_PARITY_SCENARIOS,
  createPlayerMovementParityPlan,
  recordPlayerMovement,
  runPlayerMovementParityScenario,
  summarizePlayerMovement
} from './player_movement_parity_scenarios.mjs'
import { offlineUuid } from './runner.mjs'

for (const kind of Object.keys(PLAYER_MOVEMENT_PARITY_SCENARIOS)) {
  test(`createPlayerMovementParityPlan covers ${kind}`, () => {
    const plan = createPlayerMovementParityPlan(kind)
    assert.equal(plan.name, `mineflayer-player-${kind.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`)
    assert.equal(plan.uuid, offlineUuid(plan.username))
    assert.deepEqual(plan.steps, PLAYER_MOVEMENT_PARITY_SCENARIOS[kind])
  })

  test(`runPlayerMovementParityScenario validates all ${kind} steps`, async () => {
    const result = await runPlayerMovementParityScenario(kind, {
      probe: async plan => ({
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { comparedToVanilla: true }]))
      })
    })
    assert.equal(result.summary.ok, true)
  })
}

test('summarizePlayerMovement fails on incomplete movement evidence', () => {
  const plan = createPlayerMovementParityPlan('movement')
  const evidence = { timeline: [] }
  recordPlayerMovement(evidence, plan.steps[0])
  assert.equal(summarizePlayerMovement(evidence, plan).ok, false)
})

test('runPlayerMovementParityScenario reports missing teleport confirmation evidence', async () => {
  await assert.rejects(() => runPlayerMovementParityScenario('teleportPositionConfirm', {
    probe: async plan => ({
      steps: Object.fromEntries(plan.steps.slice(0, -1).map(step => [step, true]))
    })
  }), /stale-teleport-confirmation/)
})
