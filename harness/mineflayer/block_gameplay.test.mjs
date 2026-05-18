import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createBlockGameplayPlan,
  summarizeBlockGameplay,
  planBreakBareHand,
  planBreakCorrectTool,
  planPlaceOnFace,
  assertReachEnforcement,
  assertSpawnProtectionBlock
} from './block_gameplay.mjs'

test('createBlockGameplayPlan has all required steps for offline-mode block parity', () => {
  const plan = createBlockGameplayPlan({ username: 'TestBot' })
  assert.equal(plan.name, 'mineflayer-block-gameplay')
  assert.equal(plan.serverProperties['online-mode'], 'false')
  assert.equal(plan.serverProperties['spawn-protection'], '0')
  assert.deepEqual(plan.steps, [
    'break-block-bare-hand',
    'break-block-correct-tool',
    'place-block-on-face',
    'placement-denied-too-far',
    'placement-denied-spawn-protection',
    'block-update-ack-received'
  ])
})

test('summarizeBlockGameplay returns ok when all steps have evidence', () => {
  const plan = createBlockGameplayPlan()
  const session = {
    timeline: [
      { name: 'block', summary: ['block.break.bare_hand', {}] },
      { name: 'block', summary: ['block.break.correct_tool', {}] },
      { name: 'block', summary: ['block.place.face', {}] },
      { name: 'block', summary: ['block.placement.denied.too_far', {}] },
      { name: 'block', summary: ['block.placement.denied.spawn_protection', {}] },
      { name: 'block', summary: ['block.update.ack', {}] }
    ]
  }
  const summary = summarizeBlockGameplay(session, plan)
  assert.equal(summary.ok, true)
  assert.ok(Object.values(summary.steps).every(Boolean))
})

test('summarizeBlockGameplay fails when steps are missing', () => {
  const plan = createBlockGameplayPlan()
  const session = { timeline: [{ name: 'block', summary: ['block.break.bare_hand', {}] }] }
  const summary = summarizeBlockGameplay(session, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.steps['break-block-bare-hand'], true)
  assert.equal(summary.steps['break-block-correct-tool'], false)
})

test('planBreakBareHand and planBreakCorrectTool produce expected plan shape', () => {
  const bare = planBreakBareHand('minecraft:dirt')
  assert.equal(bare.action, 'block.break.bare_hand')
  assert.deepEqual(bare.expectedDrops, [])

  const tool = planBreakCorrectTool('minecraft:stone', 'minecraft:diamond_pickaxe')
  assert.equal(tool.action, 'block.break.correct_tool')
  assert.equal(tool.toolId, 'minecraft:diamond_pickaxe')
  assert.deepEqual(tool.expectedDrops, ['minecraft:stone'])
})

test('planPlaceOnFace records face hit orientation', () => {
  const plan = planPlaceOnFace('minecraft:furnace', 'north', 'facing=north')
  assert.equal(plan.action, 'block.place.face')
  assert.equal(plan.targetFace, 'north')
  assert.equal(plan.expectedFacingProp, 'facing=north')
})

test('assertReachEnforcement detects too-far placements beyond 5.0 survival reach', () => {
  const within = assertReachEnforcement(4.9, 5.0, false)
  assert.equal(within.action, 'block.placement.ok')

  const tooFar = assertReachEnforcement(5.1, 5.0, true)
  assert.equal(tooFar.action, 'block.placement.denied.too_far')
  assert.equal(tooFar.denied, true)
})

test('assertSpawnProtectionBlock models spawn-protection denial correctly', () => {
  const denied = assertSpawnProtectionBlock(5.0, 16, true)
  assert.equal(denied.action, 'block.placement.denied.spawn_protection')

  const allowed = assertSpawnProtectionBlock(20.0, 16, false)
  assert.equal(allowed.action, 'block.placement.ok')
})
