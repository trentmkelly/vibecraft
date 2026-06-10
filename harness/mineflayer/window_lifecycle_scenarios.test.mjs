import test from 'node:test'
import assert from 'node:assert/strict'
import {
  WINDOW_LIFECYCLE_MENUS,
  WINDOW_LIFECYCLE_STEPS,
  createWindowLifecyclePlan,
  recordWindowLifecycleEvent,
  runWindowLifecycleScenario,
  summarizeWindowLifecycleEvidence
} from './window_lifecycle_scenarios.mjs'
import { offlineUuid } from './runner.mjs'

for (const menu of Object.keys(WINDOW_LIFECYCLE_MENUS)) {
  test(`createWindowLifecyclePlan covers ${menu}`, () => {
    const plan = createWindowLifecyclePlan(menu)
    assert.equal(plan.name, `mineflayer-window-lifecycle-${menu.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`)
    assert.equal(plan.menu, menu)
    assert.equal(plan.mode, 'offline')
    assert.equal(plan.auth, 'offline')
    assert.equal(plan.uuid, offlineUuid(plan.username))
    assert.equal(plan.slotCount, WINDOW_LIFECYCLE_MENUS[menu].slotCount)
    if (plan.openOnJoin) {
      assert.ok(!plan.steps.includes('open-window'))
    } else {
      assert.deepEqual(plan.steps, WINDOW_LIFECYCLE_STEPS)
    }
  })

  test(`runWindowLifecycleScenario validates ${menu} steps when probe reports them`, async () => {
    const result = await runWindowLifecycleScenario(menu, {
      probe: async plan => ({
        slotCount: plan.slotCount,
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { observed: true }]))
      })
    })
    assert.equal(result.summary.ok, true)
    for (const step of result.plan.steps) {
      assert.equal(result.summary.steps[step], true)
    }
  })
}

test('createWindowLifecyclePlan throws on unknown menu', () => {
  assert.throws(() => createWindowLifecyclePlan('jukebox'), /Unknown window lifecycle menu/)
})

test('summarizeWindowLifecycleEvidence fails on missing evidence', () => {
  const plan = createWindowLifecyclePlan('chest')
  const evidence = { timeline: [] }
  recordWindowLifecycleEvent(evidence, plan.steps[0])
  const summary = summarizeWindowLifecycleEvidence(evidence, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.steps[plan.steps[0]], true)
})

test('runWindowLifecycleScenario throws when slot count differs from menu definition', async () => {
  await assert.rejects(
    () => runWindowLifecycleScenario('chest', {
      probe: async plan => ({
        slotCount: plan.slotCount - 1, // wrong number → must fail closed
        steps: Object.fromEntries(plan.steps.map(step => [step, true]))
      })
    }),
    /Slot count mismatch/
  )
})

test('default probe rejects until a server fixture wires the menus', async () => {
  await assert.rejects(
    () => runWindowLifecycleScenario('chest'),
    /requires a live VibeCraft server/
  )
})

test('player inventory plan omits the explicit open-window step', () => {
  const plan = createWindowLifecyclePlan('playerInventory')
  assert.equal(plan.openOnJoin, true)
  assert.ok(!plan.steps.includes('open-window'))
  assert.ok(plan.steps.includes('observe-container-set-content'))
  assert.ok(plan.steps.includes('click-slot'))
})
