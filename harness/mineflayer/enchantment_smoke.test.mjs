import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createEnchantmentSmokePlan,
  summarizeEnchantmentSmoke,
  ENCHANTED_SMOKE_ITEMS,
  buildGiveCommand,
  buildGiveCommandForTarget,
  recordEnchantEvent,
  runEnchantmentSmoke
} from './enchantment_smoke.mjs'

test('createEnchantmentSmokePlan covers all 5 enchantment smoke steps', () => {
  const plan = createEnchantmentSmokePlan()
  assert.equal(plan.name, 'mineflayer-enchantment-smoke')
  assert.equal(plan.steps.length, 5)
  assert.ok(plan.steps.includes('give-enchanted-sword-no-registry-error'))
  assert.ok(plan.steps.includes('give-enchanted-pickaxe-no-registry-error'))
  assert.ok(plan.steps.includes('give-enchanted-boots-no-registry-error'))
  assert.ok(plan.steps.includes('tooltip-decode-no-component-error'))
  assert.ok(plan.steps.includes('enchant-tag-lookup-no-missing-tag'))
})

test('createEnchantmentSmokePlan uses creative mode for /give access', () => {
  const plan = createEnchantmentSmokePlan()
  assert.equal(plan.serverProperties.gamemode, 'creative')
})

test('summarizeEnchantmentSmoke returns ok when all steps evidenced', () => {
  const plan = createEnchantmentSmokePlan()
  const actionMap = {
    'give-enchanted-sword-no-registry-error': 'enchant.give.sword',
    'give-enchanted-pickaxe-no-registry-error': 'enchant.give.pickaxe',
    'give-enchanted-boots-no-registry-error': 'enchant.give.boots',
    'tooltip-decode-no-component-error': 'enchant.tooltip.decode_ok',
    'enchant-tag-lookup-no-missing-tag': 'enchant.tag.lookup_ok'
  }
  const timeline = plan.steps.map(step => ({
    name: 'enchant', summary: [actionMap[step], {}]
  }))
  assert.equal(summarizeEnchantmentSmoke({ timeline }, plan).ok, true)
})

test('summarizeEnchantmentSmoke fails with empty timeline', () => {
  const plan = createEnchantmentSmokePlan()
  assert.equal(summarizeEnchantmentSmoke({ timeline: [] }, plan).ok, false)
})

test('ENCHANTED_SMOKE_ITEMS covers sword, pickaxe, boots with valid enchantments', () => {
  assert.equal(ENCHANTED_SMOKE_ITEMS.length, 3)
  const ids = ENCHANTED_SMOKE_ITEMS.map(i => i.itemId)
  assert.ok(ids.includes('minecraft:diamond_sword'))
  assert.ok(ids.includes('minecraft:diamond_pickaxe'))
  assert.ok(ids.includes('minecraft:diamond_boots'))
  for (const item of ENCHANTED_SMOKE_ITEMS) {
    assert.ok(item.enchantments.length > 0)
    for (const e of item.enchantments) {
      assert.ok(e.id.startsWith('minecraft:'), `enchantment id ${e.id} must use minecraft: namespace`)
      assert.ok(e.level >= 1)
    }
  }
})

test('buildGiveCommand produces valid /give command string', () => {
  const sword = ENCHANTED_SMOKE_ITEMS.find(i => i.itemId === 'minecraft:diamond_sword')
  const cmd = buildGiveCommand(sword)
  assert.ok(cmd.startsWith('/give @s minecraft:diamond_sword'))
  assert.ok(cmd.includes('minecraft:sharpness'))
  assert.ok(cmd.includes('minecraft:enchantments={levels:'))
  assert.equal(
    buildGiveCommandForTarget(sword, 'EnchantBot'),
    '/give EnchantBot minecraft:diamond_sword[minecraft:enchantments={levels:{"minecraft:sharpness":5,"minecraft:looting":3,"minecraft:unbreaking":3}}] 1'
  )
})

test('recordEnchantEvent appends to session timeline', () => {
  const session = { timeline: [] }
  recordEnchantEvent(session, 'enchant.give.sword', { item: 'diamond_sword' })
  assert.equal(session.timeline.length, 1)
  assert.equal(session.timeline[0].name, 'enchant')
  assert.equal(session.timeline[0].summary[0], 'enchant.give.sword')
})

test('runEnchantmentSmoke joins, gives enchanted items, and records decode evidence', async () => {
  const writes = []
  const inventory = [
    { name: 'diamond_sword', count: 1, displayName: 'Diamond Sword', components: { 'minecraft:enchantments': {} } },
    { name: 'diamond_pickaxe', count: 1, displayName: 'Diamond Pickaxe', components: { 'minecraft:enchantments': {} } },
    { name: 'diamond_boots', count: 1, displayName: 'Diamond Boots', components: { 'minecraft:enchantments': {} } }
  ]
  let cleaned = false
  const result = await runEnchantmentSmoke({
    port: 25565,
    runObservedOfflineLogin: async () => ({
      profile: { username: 'EnchantBot' },
      timeline: [],
      bot: { inventory: { items: () => inventory } },
      server: { child: { stdin: { write: line => writes.push(line) } } },
      cleanup: async () => { cleaned = true }
    }),
    waitForSpawn: async () => {},
    timeoutMs: 100
  })

  assert.equal(result.summary.ok, true)
  assert.equal(writes.length, 3)
  assert.ok(writes.every(line => line.startsWith('give EnchantBot minecraft:')))
  assert.equal(cleaned, true)
})
