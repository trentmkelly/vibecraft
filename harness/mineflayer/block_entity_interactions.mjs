// Offline-mode block-entity interaction tests: signs, chests, furnaces, lecterns, bells,
// note blocks, campfires, cauldrons, spawners, brushable blocks.

export function createBlockEntityInteractionsPlan(options = {}) {
  return {
    name: 'mineflayer-block-entity-interactions',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'BlockEntityBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'peaceful',
      'spawn-protection': '0'
    },
    steps: [
      'sign-edit',
      'chest-open-loot',
      'furnace-insert-extract',
      'lectern-page-turn',
      'bell-ring-broadcast',
      'note-block-pitch',
      'campfire-cook',
      'cauldron-fill-empty',
      'spawner-egg-insert',
      'brushable-block-brush'
    ]
  }
}

export function summarizeBlockEntityInteractions(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => e.name === 'block_entity')
    .map(e => e.summary?.[0])
  const result = Object.fromEntries(
    plan.steps.map(step => [step, actions.includes(stepToAction(step))])
  )
  return {
    ok: Object.values(result).every(Boolean),
    steps: result
  }
}

function stepToAction(step) {
  const map = {
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
  }
  return map[step] ?? step
}

export function recordBlockEntityEvent(session, action, details = {}) {
  session.timeline.push({ name: 'block_entity', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

export function planSignEdit(lines) {
  return { action: 'block_entity.sign.edit', lines: lines ?? ['Line1', 'Line2', 'Line3', 'Line4'] }
}

export function planChestOpenLoot(expectedSlotCount) {
  return { action: 'block_entity.chest.open_loot', expectedSlotCount: expectedSlotCount ?? 27 }
}

export function planFurnaceInsertExtract(inputItem, fuelItem, expectedOutput) {
  return { action: 'block_entity.furnace.insert_extract', inputItem, fuelItem, expectedOutput }
}

export function planLecternPageTurn(bookTitle, pageCount) {
  return { action: 'block_entity.lectern.page_turn', bookTitle, pageCount }
}

export function planBellRingBroadcast(expectedBroadcastRadius) {
  return { action: 'block_entity.bell.ring_broadcast', expectedBroadcastRadius: expectedBroadcastRadius ?? 32 }
}

export function planNoteBlockPitch(expectedPitch) {
  return { action: 'block_entity.note_block.pitch', expectedPitch }
}

export function planCampfireCook(inputFood, expectedOutput) {
  return { action: 'block_entity.campfire.cook', inputFood, expectedOutput }
}

export function planCauldronFillEmpty(fluidType) {
  return { action: 'block_entity.cauldron.fill_empty', fluidType: fluidType ?? 'water' }
}

export function planSpawnerEggInsert(mobType) {
  return { action: 'block_entity.spawner.egg_insert', mobType }
}

export function planBrushableBlockBrush(expectedLootTable) {
  return { action: 'block_entity.brushable.brush', expectedLootTable }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('block_entity_interactions scenarios defined — run via the test harness')
}
