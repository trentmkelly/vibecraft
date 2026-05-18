import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createBlockDropsPlan,
  summarizeBlockDrops,
  expectNoDrops,
  expectDrops,
  expectFortuneDrops
} from './block_drops.mjs'

test('createBlockDropsPlan covers all 8 drop verification steps', () => {
  const plan = createBlockDropsPlan()
  assert.equal(plan.name, 'mineflayer-block-drops')
  assert.equal(plan.steps.length, 8)
  assert.ok(plan.steps.includes('break-bare-hand-no-drop'))
  assert.ok(plan.steps.includes('break-fortune-3-bonus'))
  assert.ok(plan.steps.includes('dotiledrops-false-no-drops'))
})

test('summarizeBlockDrops returns ok when all steps are evidenced', () => {
  const plan = createBlockDropsPlan()
  const stepToAction = {
    'break-bare-hand-no-drop': 'drops.bare_hand.none',
    'break-correct-tool-drop': 'drops.correct_tool.present',
    'break-silk-touch-block-drops': 'drops.silk_touch.block_itself',
    'break-fortune-1-bonus': 'drops.fortune.1',
    'break-fortune-2-bonus': 'drops.fortune.2',
    'break-fortune-3-bonus': 'drops.fortune.3',
    'explosion-no-drops': 'drops.explosion.none',
    'dotiledrops-false-no-drops': 'drops.gamerule.none'
  }
  const timeline = plan.steps.map(step => ({
    name: 'drops', summary: [stepToAction[step], {}]
  }))
  const summary = summarizeBlockDrops({ timeline }, plan)
  assert.equal(summary.ok, true)
})

test('summarizeBlockDrops fails with empty timeline', () => {
  const plan = createBlockDropsPlan()
  const summary = summarizeBlockDrops({ timeline: [] }, plan)
  assert.equal(summary.ok, false)
})

test('expectNoDrops records correct structure', () => {
  const result = expectNoDrops('minecraft:iron_ore', 'bare_hand')
  assert.deepEqual(result.expectedDrops, [])
  assert.equal(result.reason, 'bare_hand')
})

test('expectDrops records expected drops list', () => {
  const result = expectDrops('minecraft:stone', ['minecraft:cobblestone'])
  assert.deepEqual(result.expectedDrops, ['minecraft:cobblestone'])
})

test('expectFortuneDrops records fortune level and count range', () => {
  const result = expectFortuneDrops('minecraft:diamond_ore', 3, 1, 4)
  assert.equal(result.fortuneLevel, 3)
  assert.equal(result.minCount, 1)
  assert.equal(result.maxCount, 4)
})
