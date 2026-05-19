import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createBlockEntityInteractionsPlan,
  summarizeBlockEntityInteractions,
  planSignEdit,
  planHangingSignEdit,
  planBookInteract,
  planCommandBlockProgram,
  planSkullAnimation,
  planBannerPattern,
  planConduitPulse,
  planCandleAdjust,
  planChestOpenLoot,
  planFurnaceInsertExtract,
  planLecternPageTurn,
  planBellRingBroadcast,
  planNoteBlockPitch,
  planCampfireCook,
  planCauldronFillEmpty,
  planSpawnerEggInsert,
  planVaultOpen,
  planTrialSpawnerEggInsert,
  planCalibratedSculkSensorTest,
  planChiseledBookshelfUse,
  planBrushableBlockBrush
} from './block_entity_interactions.mjs'

test('createBlockEntityInteractionsPlan covers requested parity block entity steps', () => {
  const plan = createBlockEntityInteractionsPlan()
  assert.equal(plan.name, 'mineflayer-block-entity-interactions')
  assert.equal(plan.steps.length, 21)
  assert.deepEqual(plan.steps, [
    'chest-open-loot',
    'furnace-insert-extract',
    'sign-edit',
    'hanging-sign-edit',
    'book-interact',
    'command-block-program',
    'skull-animation',
    'banner-pattern',
    'conduit-pulse',
    'bell-ring-broadcast',
    'lectern-page-turn',
    'note-block-pitch',
    'campfire-cook',
    'cauldron-fill-empty',
    'candle-adjust',
    'spawner-egg-insert',
    'vault-open',
    'trial-spawner-egg-insert',
    'calibrated-sculk-sensor-test',
    'chiseled-bookshelf-use',
    'brushable-block-brush'
  ])
})

test('summarizeBlockEntityInteractions returns ok when all steps have evidence', () => {
  const plan = createBlockEntityInteractionsPlan()
  const timeline = plan.steps.map(step => ({
    name: 'block_entity',
    summary: [
      {
        'sign-edit': 'block_entity.sign.edit',
        'hanging-sign-edit': 'block_entity.hanging_sign.edit',
        'book-interact': 'block_entity.book.interact',
        'command-block-program': 'block_entity.command_block.program',
        'skull-animation': 'block_entity.skull.animate',
        'banner-pattern': 'block_entity.banner.pattern',
        'conduit-pulse': 'block_entity.conduit.pulse',
        'candle-adjust': 'block_entity.candle.adjust',
        'chest-open-loot': 'block_entity.chest.open_loot',
        'furnace-insert-extract': 'block_entity.furnace.insert_extract',
        'lectern-page-turn': 'block_entity.lectern.page_turn',
        'bell-ring-broadcast': 'block_entity.bell.ring_broadcast',
        'note-block-pitch': 'block_entity.note_block.pitch',
        'campfire-cook': 'block_entity.campfire.cook',
        'cauldron-fill-empty': 'block_entity.cauldron.fill_empty',
        'spawner-egg-insert': 'block_entity.spawner.egg_insert',
        'vault-open': 'block_entity.vault.open',
        'trial-spawner-egg-insert': 'block_entity.trial_spawner.egg_insert',
        'calibrated-sculk-sensor-test': 'block_entity.calibrated_sculk_sensor.test',
        'chiseled-bookshelf-use': 'block_entity.chiseled_bookshelf.use',
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
  assert.equal(planHangingSignEdit().action, 'block_entity.hanging_sign.edit')
  assert.equal(planBookInteract('My Book', 3).action, 'block_entity.book.interact')
  assert.equal(planCommandBlockProgram('say hi').action, 'block_entity.command_block.program')
  assert.equal(planSkullAnimation('dragon').action, 'block_entity.skull.animate')
  assert.equal(planBannerPattern(3).action, 'block_entity.banner.pattern')
  assert.equal(planConduitPulse().action, 'block_entity.conduit.pulse')
  assert.equal(planCandleAdjust(2).action, 'block_entity.candle.adjust')
  assert.equal(planVaultOpen().action, 'block_entity.vault.open')
  assert.equal(planTrialSpawnerEggInsert('minecraft:trial_spawner').action, 'block_entity.trial_spawner.egg_insert')
  assert.equal(planCalibratedSculkSensorTest().action, 'block_entity.calibrated_sculk_sensor.test')
  assert.equal(planChiseledBookshelfUse(3, true).action, 'block_entity.chiseled_bookshelf.use')
  assert.equal(planBrushableBlockBrush('minecraft:suspicious_sand').action, 'block_entity.brushable.brush')
})
