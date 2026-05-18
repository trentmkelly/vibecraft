import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createBlockEntityInteractionsPlan,
  summarizeBlockEntityInteractions,
  planSignEdit,
  planChestOpenLoot,
  planFurnaceInsertExtract,
  planLecternPageTurn,
  planBellRingBroadcast,
  planNoteBlockPitch,
  planCampfireCook,
  planCauldronFillEmpty,
  planSpawnerEggInsert,
  planBrushableBlockBrush
} from './block_entity_interactions.mjs'

test('createBlockEntityInteractionsPlan covers all 10 block entity interaction steps', () => {
  const plan = createBlockEntityInteractionsPlan()
  assert.equal(plan.name, 'mineflayer-block-entity-interactions')
  assert.equal(plan.steps.length, 10)
  assert.ok(plan.steps.includes('sign-edit'))
  assert.ok(plan.steps.includes('chest-open-loot'))
  assert.ok(plan.steps.includes('brushable-block-brush'))
})

test('summarizeBlockEntityInteractions returns ok when all steps have evidence', () => {
  const plan = createBlockEntityInteractionsPlan()
  const timeline = plan.steps.map(step => ({
    name: 'block_entity',
    summary: [
      {
        'sign-edit': 'block_entity.sign.edit',
        'chest-open-loot': 'block_entity.chest.open_loot',
        'furnace-insert-extract': 'block_entity.furnace.insert_extract',
        'lectern-page-turn': 'block_entity.lectern.page_turn',
        'bell-ring-broadcast': 'block_entity.bell.ring_broadcast',
        'note-block-pitch': 'block_entity.note_block.pitch',
        'campfire-cook': 'block_entity.campfire.cook',
        'cauldron-fill-empty': 'block_entity.cauldron.fill_empty',
        'spawner-egg-insert': 'block_entity.spawner.egg_insert',
        'brushable-block-brush': 'block_entity.brushable.brush'
      }[step],
      {}
    ]
  }))
  const summary = summarizeBlockEntityInteractions({ timeline }, plan)
  assert.equal(summary.ok, true)
})

test('summarizeBlockEntityInteractions fails with empty timeline', () => {
  const plan = createBlockEntityInteractionsPlan()
  const summary = summarizeBlockEntityInteractions({ timeline: [] }, plan)
  assert.equal(summary.ok, false)
})

test('planSignEdit defaults to 4 lines', () => {
  const p = planSignEdit()
  assert.equal(p.lines.length, 4)
})

test('planChestOpenLoot defaults to 27 slots', () => {
  const p = planChestOpenLoot()
  assert.equal(p.expectedSlotCount, 27)
})

test('planFurnaceInsertExtract records correct items', () => {
  const p = planFurnaceInsertExtract('minecraft:raw_iron', 'minecraft:coal', 'minecraft:iron_ingot')
  assert.equal(p.inputItem, 'minecraft:raw_iron')
  assert.equal(p.fuelItem, 'minecraft:coal')
  assert.equal(p.expectedOutput, 'minecraft:iron_ingot')
})

test('planBellRingBroadcast defaults to radius 32', () => {
  const p = planBellRingBroadcast()
  assert.equal(p.expectedBroadcastRadius, 32)
})

test('all plan functions produce correct action strings', () => {
  assert.equal(planLecternPageTurn('My Book', 5).action, 'block_entity.lectern.page_turn')
  assert.equal(planNoteBlockPitch(12).action, 'block_entity.note_block.pitch')
  assert.equal(planCampfireCook('minecraft:beef', 'minecraft:cooked_beef').action, 'block_entity.campfire.cook')
  assert.equal(planCauldronFillEmpty('water').action, 'block_entity.cauldron.fill_empty')
  assert.equal(planSpawnerEggInsert('minecraft:zombie').action, 'block_entity.spawner.egg_insert')
  assert.equal(planBrushableBlockBrush('minecraft:suspicious_sand').action, 'block_entity.brushable.brush')
})
