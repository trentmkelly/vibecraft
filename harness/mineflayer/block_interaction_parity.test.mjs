import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createBlockInteractionParityPlan,
  summarizeBlockInteractionParity,
  planRightClickUse,
  planSneakUseBypass,
  planPredictionRollback,
  planBlockEntityTagAfterUse,
  planNeighborShapeUpdate
} from './block_interaction_parity.mjs'

test('createBlockInteractionParityPlan covers all 5 interaction parity steps', () => {
  const plan = createBlockInteractionParityPlan({ username: 'TestBot' })
  assert.equal(plan.name, 'mineflayer-block-interaction-parity')
  assert.equal(plan.mode, 'offline')
  assert.deepEqual(plan.steps, [
    'right-click-block-use',
    'sneak-use-bypass',
    'client-prediction-rollback',
    'block-entity-tag-after-use',
    'neighbor-shape-update-after-placement'
  ])
})

test('summarizeBlockInteractionParity returns ok when all steps have evidence', () => {
  const plan = createBlockInteractionParityPlan()
  const session = {
    timeline: [
      { name: 'interaction', summary: ['interaction.right_click_use', {}] },
      { name: 'interaction', summary: ['interaction.sneak_use_bypass', {}] },
      { name: 'interaction', summary: ['interaction.prediction_rollback', {}] },
      { name: 'interaction', summary: ['interaction.block_entity_tag', {}] },
      { name: 'interaction', summary: ['interaction.neighbor_shape_update', {}] }
    ]
  }
  const summary = summarizeBlockInteractionParity(session, plan)
  assert.equal(summary.ok, true)
})

test('summarizeBlockInteractionParity fails on missing evidence', () => {
  const plan = createBlockInteractionParityPlan()
  const summary = summarizeBlockInteractionParity({ timeline: [] }, plan)
  assert.equal(summary.ok, false)
  assert.ok(Object.values(summary.steps).every(v => v === false))
})

test('planRightClickUse records block id and expected InteractionResult', () => {
  const p = planRightClickUse('minecraft:crafting_table', 'SUCCESS')
  assert.equal(p.action, 'interaction.right_click_use')
  assert.equal(p.blockId, 'minecraft:crafting_table')
  assert.equal(p.expectedResult, 'SUCCESS')
})

test('planSneakUseBypass records sneak-bypass scenario fields', () => {
  const p = planSneakUseBypass('minecraft:chest', 'minecraft:stick')
  assert.equal(p.action, 'interaction.sneak_use_bypass')
  assert.equal(p.blockId, 'minecraft:chest')
  assert.equal(p.heldItemId, 'minecraft:stick')
  assert.ok(p.description.includes('sneak'))
})

test('planPredictionRollback records rollback scenario', () => {
  const pos = { x: 0, y: 64, z: 0 }
  const p = planPredictionRollback(pos, 'minecraft:stone')
  assert.equal(p.action, 'interaction.prediction_rollback')
  assert.deepEqual(p.targetPos, pos)
  assert.ok(p.description.includes('corrective'))
})

test('planBlockEntityTagAfterUse and planNeighborShapeUpdate have correct shapes', () => {
  const tag = planBlockEntityTagAfterUse('minecraft:furnace', ['Items', 'BurnTime'])
  assert.equal(tag.action, 'interaction.block_entity_tag')
  assert.deepEqual(tag.expectedNbtKeys, ['Items', 'BurnTime'])

  const shape = planNeighborShapeUpdate('minecraft:redstone_wire', { signalChanged: true })
  assert.equal(shape.action, 'interaction.neighbor_shape_update')
  assert.deepEqual(shape.expectedNeighborUpdate, { signalChanged: true })
})
