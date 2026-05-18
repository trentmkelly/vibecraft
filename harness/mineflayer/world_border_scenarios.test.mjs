import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createWorldBorderScenariosPlan,
  summarizeWorldBorderScenarios,
  computeBorderDamage,
  planBorderInit,
  planBorderLerp,
  planBorderWarning,
  planBorderDamage
} from './world_border_scenarios.mjs'

test('createWorldBorderScenariosPlan covers all 7 world border steps', () => {
  const plan = createWorldBorderScenariosPlan()
  assert.equal(plan.name, 'mineflayer-world-border-scenarios')
  assert.equal(plan.steps.length, 7)
  assert.ok(plan.steps.includes('init-border-size-center-packet'))
  assert.ok(plan.steps.includes('damage-outside-buffer'))
  assert.ok(plan.steps.includes('command-update-sends-packets'))
})

test('summarizeWorldBorderScenarios returns ok when all steps evidenced', () => {
  const plan = createWorldBorderScenariosPlan()
  const actionMap = {
    'init-border-size-center-packet': 'border.init.packet',
    'lerp-new-size-with-time': 'border.lerp.size_time',
    'warning-distance-packet': 'border.warning.distance',
    'warning-time-packet': 'border.warning.time',
    'damage-outside-buffer': 'border.damage.outside_buffer',
    'movement-clamped-at-border': 'border.movement.clamped',
    'command-update-sends-packets': 'border.command.update'
  }
  const timeline = plan.steps.map(step => ({
    name: 'border', summary: [actionMap[step], {}]
  }))
  assert.equal(summarizeWorldBorderScenarios({ timeline }, plan).ok, true)
})

test('summarizeWorldBorderScenarios fails with empty timeline', () => {
  const plan = createWorldBorderScenariosPlan()
  assert.equal(summarizeWorldBorderScenarios({ timeline: [] }, plan).ok, false)
})

test('computeBorderDamage is 0.2 per block outside buffer', () => {
  // Parity test: rate is 0.2 * distance outside buffer
  assert.equal(computeBorderDamage(0), 0)      // at buffer edge
  assert.equal(computeBorderDamage(1), 0.2)    // 1 block outside
  assert.equal(computeBorderDamage(5), 1.0)    // 5 blocks outside
  assert.equal(computeBorderDamage(-1), 0)     // inside buffer → 0
})

test('planBorderInit records size and center', () => {
  const p = planBorderInit(60000000, 0, 0)
  assert.equal(p.action, 'border.init.packet')
  assert.equal(p.expectedSize, 60000000)
  assert.equal(p.expectedCenterX, 0)
  assert.equal(p.expectedCenterZ, 0)
})

test('planBorderLerp detects lerp conditions', () => {
  const lerping = planBorderLerp(1000, 500, 60000)
  assert.equal(lerping.expectLerping, true)
  assert.equal(lerping.oldSize, 1000)
  assert.equal(lerping.newSize, 500)

  const stationary = planBorderLerp(1000, 1000, 0)
  assert.equal(stationary.expectLerping, false)
})

test('planBorderWarning produces both distance and time action shapes', () => {
  const { distanceAction, timeAction } = planBorderWarning(5, 15)
  assert.equal(distanceAction.action, 'border.warning.distance')
  assert.equal(distanceAction.warningDistanceBlocks, 5)
  assert.equal(timeAction.action, 'border.warning.time')
  assert.equal(timeAction.warningTimeSecs, 15)
})

test('planBorderDamage calculates expected damage at test distance', () => {
  const p = planBorderDamage(5, 0.2, 10)
  assert.equal(p.expectedDamage, 2.0)  // 0.2 * 10
  assert.equal(p.bufferBlocks, 5)
  assert.equal(p.testDistanceOutside, 10)
})
