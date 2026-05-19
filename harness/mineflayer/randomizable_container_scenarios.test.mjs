import test from 'node:test'
import assert from 'node:assert/strict'
import {
  RANDOMIZABLE_CONTAINER_TYPES,
  createRandomizableContainerPlan,
  planRandomizableContainerCase,
  recordRandomizableContainerEvent,
  summarizeRandomizableContainerRun
} from './randomizable_container_scenarios.mjs'

test('createRandomizableContainerPlan covers all requested container reconnect phases', () => {
  const plan = createRandomizableContainerPlan()
  assert.equal(plan.name, 'mineflayer-randomizable-containers')
  assert.deepEqual(plan.containers, [
    'chest',
    'barrel',
    'dispenser',
    'dropper',
    'shulker_box'
  ])
  assert.equal(plan.steps.length, RANDOMIZABLE_CONTAINER_TYPES.length * 2)
  for (const container of RANDOMIZABLE_CONTAINER_TYPES) {
    assert.ok(plan.steps.includes(`${container}:open-before-reconnect`))
    assert.ok(plan.steps.includes(`${container}:open-after-reconnect`))
  }
  assert.equal(plan.assertions.lootRealization, 'exactly-once')
  assert.equal(plan.assertions.customName, 'visible-on-open')
  assert.equal(plan.assertions.lock, 'enforced-before-open')
  assert.equal(plan.assertions.comparatorContents, 'matches-realized-inventory')
})

test('planRandomizableContainerCase records loot, lock, name, comparator, and reconnect actions', () => {
  const scenario = planRandomizableContainerCase('shulker_box', {
    lootTable: 'minecraft:chests/trial_chambers/reward',
    customName: 'Reward Box',
    lockKey: 'trial_key'
  })
  assert.equal(scenario.container, 'shulker_box')
  assert.equal(scenario.lootTable, 'minecraft:chests/trial_chambers/reward')
  assert.equal(scenario.customName, 'Reward Box')
  assert.equal(scenario.lockKey, 'trial_key')
  assert.equal(scenario.beforeReconnectAction, 'randomizable_container.shulker_box.before_reconnect')
  assert.equal(scenario.afterReconnectAction, 'randomizable_container.shulker_box.after_reconnect')
  assert.deepEqual(scenario.expected, {
    realizationCount: 1,
    lockedOpenDenied: true,
    unlockedOpenAllowed: true,
    customNameVisible: true,
    comparatorMatchesContents: true
  })
})

test('planRandomizableContainerCase rejects unsupported containers', () => {
  assert.throws(
    () => planRandomizableContainerCase('furnace'),
    /Unsupported randomizable container/
  )
})

test('summarizeRandomizableContainerRun requires before and after reconnect evidence', () => {
  const plan = createRandomizableContainerPlan({ containers: ['chest', 'barrel'] })
  const session = { timeline: [] }
  for (const container of plan.containers) {
    recordRandomizableContainerEvent(
      session,
      `randomizable_container.${container}.before_reconnect`,
      { realizationCount: 1 }
    )
    recordRandomizableContainerEvent(
      session,
      `randomizable_container.${container}.after_reconnect`,
      { realizationCount: 1 }
    )
  }

  assert.deepEqual(summarizeRandomizableContainerRun(session, plan), {
    ok: true,
    steps: {
      'chest:open-before-reconnect': true,
      'chest:open-after-reconnect': true,
      'barrel:open-before-reconnect': true,
      'barrel:open-after-reconnect': true
    }
  })
})

test('summarizeRandomizableContainerRun fails when reconnect evidence is missing', () => {
  const plan = createRandomizableContainerPlan({ containers: ['dropper'] })
  const session = { timeline: [] }
  recordRandomizableContainerEvent(
    session,
    'randomizable_container.dropper.before_reconnect',
    { realizationCount: 1 }
  )

  const summary = summarizeRandomizableContainerRun(session, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.steps['dropper:open-before-reconnect'], true)
  assert.equal(summary.steps['dropper:open-after-reconnect'], false)
})
