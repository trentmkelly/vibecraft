// Offline-mode block-entity interaction tests: signs, hanging signs, books,
// command blocks, skulls, banners, conduits, bells, campfires, candles,
// cauldrons, note blocks, spawners, vaults, trial spawners,
// calibrated sculk sensors, chiseled bookshelves, and brushable blocks.

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

export function planHangingSignEdit(lines) {
  return { action: 'block_entity.hanging_sign.edit', lines: lines ?? ['Top', 'Line 2', 'Line 3', 'Bottom'] }
}

export function planBookInteract(bookTitle, pageCount) {
  return { action: 'block_entity.book.interact', bookTitle: bookTitle ?? 'My Book', pageCount: pageCount ?? 1 }
}

export function planCommandBlockProgram(command) {
  return { action: 'block_entity.command_block.program', command: command ?? 'say parity-check' }
}

export function planSkullAnimation(kind) {
  return { action: 'block_entity.skull.animate', kind: kind ?? 'dragon' }
}

export function planBannerPattern(patternCount) {
  return { action: 'block_entity.banner.pattern', patternCount: patternCount ?? 4 }
}

export function planConduitPulse() {
  return { action: 'block_entity.conduit.pulse', expectMonstersToRepel: true }
}

export function planCandleAdjust(level) {
  return { action: 'block_entity.candle.adjust', level: level ?? 1 }
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

export function planVaultOpen() {
  return { action: 'block_entity.vault.open' }
}

export function planTrialSpawnerEggInsert(mobType) {
  return { action: 'block_entity.trial_spawner.egg_insert', mobType: mobType ?? 'minecraft:trial_spawner' }
}

export function planCalibratedSculkSensorTest() {
  return { action: 'block_entity.calibrated_sculk_sensor.test', expectedFrequency: 5 }
}

export function planChiseledBookshelfUse(slot, occupied) {
  return { action: 'block_entity.chiseled_bookshelf.use', slot: slot ?? 0, occupied: Boolean(occupied) }
}

export function planBrushableBlockBrush(expectedLootTable) {
  return { action: 'block_entity.brushable.brush', expectedLootTable }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('block_entity_interactions scenarios defined — run via the test harness')
}
