import test from 'node:test'
import assert from 'node:assert/strict'
import {
  FIRST_TICK_ACTION_STEPS,
  createFirstTickActionPlan,
  recordFirstTick,
  runFirstTickActions,
  summarizeFirstTickActions
} from './first_tick_actions.mjs'

test('createFirstTickActionPlan covers immediate play-state actions', () => {
  const plan = createFirstTickActionPlan()
  assert.equal(plan.name, 'mineflayer-first-tick-actions')
  assert.deepEqual(plan.steps, FIRST_TICK_ACTION_STEPS)
  assert.ok(plan.steps.includes('movement'))
  assert.ok(plan.steps.includes('chat'))
  assert.ok(plan.steps.includes('command-suggestion'))
  assert.ok(plan.steps.includes('inventory-window'))
  assert.ok(plan.steps.includes('block-look'))
})

test('summarizeFirstTickActions requires every action and no race failure', () => {
  const plan = createFirstTickActionPlan()
  const session = { timeline: [] }
  for (const step of plan.steps) recordFirstTick(session, step)
  assert.equal(summarizeFirstTickActions(session, plan).ok, true)

  session.timeline = session.timeline.filter(event => event.summary[0] !== 'block-look')
  const summary = summarizeFirstTickActions(session, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.steps['block-look'], false)
})

test('runFirstTickActions sends all immediate actions after spawn', async () => {
  let cleaned = false
  const called = []
  const result = await runFirstTickActions({
    port: 25565,
    runObservedOfflineLogin: async () => ({
      timeline: [],
      bot: {},
      cleanup: async () => { cleaned = true }
    }),
    waitForSpawn: async () => called.push('spawn'),
    sendMovement: async () => called.push('movement'),
    sendChat: async () => called.push('chat'),
    sendCommandSuggestion: async () => {
      called.push('command')
      return ['list']
    },
    sendInventoryAction: async () => {
      called.push('inventory')
      return { windowId: 0 }
    },
    sendBlockLook: async () => {
      called.push('block-look')
      return { x: 0, y: 79, z: 0 }
    }
  })

  assert.deepEqual(called, ['spawn', 'movement', 'chat', 'command', 'inventory', 'block-look'])
  assert.equal(result.summary.ok, true)
  assert.equal(cleaned, true)
})

test('runFirstTickActions fails when a race disconnect is observed', async () => {
  await assert.rejects(() => runFirstTickActions({
    port: 25565,
    runObservedOfflineLogin: async () => ({
      timeline: [{ name: 'kicked', summary: ['race'] }],
      bot: {},
      cleanup: async () => {}
    }),
    waitForSpawn: async () => {},
    sendMovement: async () => {},
    sendChat: async () => {},
    sendCommandSuggestion: async () => [],
    sendInventoryAction: async () => ({ windowId: 0 }),
    sendBlockLook: async () => ({ x: 0, y: 79, z: 0 })
  }), /race failure/)
})
