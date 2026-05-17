import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createChestPlacement,
  createFishingLoopControl,
  createInventorySetup,
  createItemEntityCollection,
  createLootEconomyFixture,
  createMobPlacement,
  createPostReconnectSnapshot,
  createToolEnchantmentSetup,
  createVillagerOfferCapture
} from './action_fixtures.mjs'

test('createInventorySetup builds deterministic clear and slot replacement commands', () => {
  const setup = createInventorySetup({ username: 'Steve' }, [
    { slot: 'hotbar.0', item: 'stone', count: 64 },
    { slot: 'inventory.9', item: 'minecraft:emerald', count: 3 }
  ])
  assert.deepEqual(setup.commands, [
    '/clear Steve',
    '/item replace entity Steve hotbar.0 with minecraft:stone 64',
    '/item replace entity Steve inventory.9 with minecraft:emerald 3'
  ])
})

test('createToolEnchantmentSetup produces sorted enchantment components', () => {
  const setup = createToolEnchantmentSetup({
    username: 'Alex',
    item: 'diamond_pickaxe',
    enchantments: { efficiency: 5, fortune: 3 }
  })
  assert.equal(setup.enchantments[0].id, 'minecraft:efficiency')
  assert.equal(
    setup.command,
    '/item replace entity Alex weapon.mainhand with minecraft:diamond_pickaxe[minecraft:enchantments={levels:{"minecraft:efficiency":5,"minecraft:fortune":3}}] 1'
  )
})

test('createMobPlacement and createChestPlacement build setup commands', () => {
  assert.equal(
    createMobPlacement({ type: 'zombie', pos: { x: 1, y: 65, z: 2 }, nbt: { NoAI: 1 } }).command,
    '/summon minecraft:zombie 1 65 2 {"NoAI":1}'
  )
  assert.deepEqual(
    createChestPlacement({
      pos: { x: 3, y: 64, z: 4 },
      items: [{ slotIndex: 5, item: 'diamond', count: 2 }]
    }).commands,
    [
      '/setblock 3 64 4 minecraft:chest replace',
      '/item replace block 3 64 4 container.5 with minecraft:diamond 2'
    ]
  )
})

test('createVillagerOfferCapture models merchant setup and expected capture shape', () => {
  const fixture = createVillagerOfferCapture({
    profession: 'toolsmith',
    offers: [{ buy: { item: 'emerald', count: 4 }, sell: { item: 'iron_pickaxe', count: 1 } }]
  })
  assert.equal(fixture.capture.windowType, 'minecraft:merchant')
  assert.equal(fixture.capture.expectedOfferCount, 1)
  assert.match(fixture.command, /profession:"minecraft:toolsmith"/)
  assert.match(fixture.command, /sell:\{id:"minecraft:iron_pickaxe",count:1\}/)
})

test('fishing and item entity collection fixtures keep deterministic expectations', () => {
  assert.deepEqual(createFishingLoopControl({
    casts: 2,
    expectedLoot: [{ item: 'cod', count: 1 }]
  }), {
    kind: 'fishingLoop',
    casts: 2,
    timeoutMs: 30000,
    reelInOnBite: true,
    stopOnFirstCatch: false,
    expectedLoot: [{ item: 'minecraft:cod', count: 1 }]
  })
  const item = createItemEntityCollection({ stack: { item: 'emerald', count: 2 } })
  assert.equal(item.command, '/summon minecraft:item 0 64 0 {Item:{id:"minecraft:emerald",count:2}}')
  assert.deepEqual(item.expectedInventoryDelta, { 'minecraft:emerald': 2 })
})

test('createPostReconnectSnapshot captures inventory, timeline, and packet artifacts', () => {
  const snapshot = createPostReconnectSnapshot({
    profile: { username: 'Bot' },
    uuid: 'uuid',
    bot: {
      heldItem: { name: 'stick', count: 1, displayName: 'Stick' },
      inventory: { slots: [null, { name: 'emerald', count: 4, displayName: 'Emerald' }] },
      entity: { position: { x: 1.5, y: 64, z: -2.5 } }
    },
    timeline: [{ name: 'spawn', at: 1, summary: [] }],
    packetTrace: [{ name: 'login', state: 'play', at: 1, keys: ['entityId'] }]
  })
  assert.deepEqual(snapshot.inventory, [{ slot: 1, name: 'emerald', count: 4, displayName: 'Emerald' }])
  assert.deepEqual(snapshot.position, { x: 1.5, y: 64, z: -2.5 })
  assert.deepEqual(snapshot.packetTrace, [{ name: 'login', state: 'play', keys: ['entityId'] }])
})

test('createLootEconomyFixture bundles all reusable action fixtures', () => {
  const fixture = createLootEconomyFixture({
    profile: { username: 'TraderBot' },
    inventory: [{ item: 'emerald', count: 8 }],
    chest: { items: [{ item: 'bread', count: 3 }] }
  })
  assert.equal(fixture.name, 'loot-economy')
  assert.equal(fixture.inventory.username, 'TraderBot')
  assert.equal(fixture.tool.username, 'TraderBot')
  assert.equal(fixture.chest.items[0].item, 'minecraft:bread')
  assert.equal(fixture.villager.kind, 'villagerOffers')
})
