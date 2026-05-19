import test from 'node:test'
import assert from 'node:assert/strict'
import {
  FIRST_ACTION_MATRIX_STEPS,
  createFirstActionMatrixPlan,
  recordFirstAction,
  runFirstActionMatrix,
  summarizeFirstActionMatrix
} from './first_action_matrix.mjs'

test('createFirstActionMatrixPlan covers movement/chat/command/inventory/dig/place', () => {
  const plan = createFirstActionMatrixPlan()
  assert.equal(plan.name, 'mineflayer-first-action-matrix')
  assert.deepEqual(plan.steps, FIRST_ACTION_MATRIX_STEPS)
  for (const step of ['movement', 'chat', 'command-suggestion', 'inventory-click', 'block-dig', 'block-place']) {
    assert.ok(plan.steps.includes(step))
  }
})

test('summarizeFirstActionMatrix requires every matrix action', () => {
  const plan = createFirstActionMatrixPlan()
  const session = { timeline: [] }
  for (const step of plan.steps) recordFirstAction(session, step)
  assert.equal(summarizeFirstActionMatrix(session, plan).ok, true)

  session.timeline = session.timeline.filter(event => event.summary[0] !== 'block-place')
  const summary = summarizeFirstActionMatrix(session, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.steps['block-place'], false)
})

test('runFirstActionMatrix executes matrix actions in order', async () => {
  const called = []
  let cleaned = false
  const result = await runFirstActionMatrix({
    port: 25565,
    runObservedOfflineLogin: async () => ({
      timeline: [],
      bot: {},
      cleanup: async () => { cleaned = true }
    }),
    waitForSpawn: async () => called.push('spawn'),
    doMovement: async () => called.push('movement'),
    doChat: async () => called.push('chat'),
    doCommandSuggestion: async () => {
      called.push('command')
      return ['list']
    },
    doInventoryClick: async () => {
      called.push('inventory')
      return { ok: true }
    },
    doBlockDig: async () => {
      called.push('dig')
      return { ok: true }
    },
    doBlockPlace: async () => {
      called.push('place')
      return { ok: true }
    }
  })

  assert.deepEqual(called, ['spawn', 'movement', 'chat', 'command', 'inventory', 'dig', 'place'])
  assert.equal(result.summary.ok, true)
  assert.equal(cleaned, true)
})

test('runFirstActionMatrix rejects race-condition disconnects', async () => {
  await assert.rejects(() => runFirstActionMatrix({
    port: 25565,
    runObservedOfflineLogin: async () => ({
      timeline: [{ name: 'kicked', summary: ['race'] }],
      bot: {},
      cleanup: async () => {}
    }),
    waitForSpawn: async () => {},
    doMovement: async () => {},
    doChat: async () => {},
    doCommandSuggestion: async () => [],
    doInventoryClick: async () => ({}),
    doBlockDig: async () => ({}),
    doBlockPlace: async () => ({})
  }), /First-action matrix failure/)
})
