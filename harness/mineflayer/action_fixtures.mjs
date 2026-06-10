export function createInventorySetup(profile, slots = []) {
  const username = profile.username ?? profile
  const normalizedSlots = slots.map(normalizeSlot)
  return {
    kind: 'inventory',
    username,
    slots: normalizedSlots,
    commands: [
      `/clear ${username}`,
      ...normalizedSlots.map(slot => `/item replace entity ${username} ${slot.slot} with ${slot.item} ${slot.count}`)
    ]
  }
}

export function createToolEnchantmentSetup(options = {}) {
  const item = namespaced(options.item ?? 'minecraft:diamond_pickaxe')
  const count = options.count ?? 1
  const slot = options.slot ?? 'weapon.mainhand'
  const enchantments = Object.entries(options.enchantments ?? {})
    .map(([id, level]) => ({ id: namespaced(id), level }))
    .sort((left, right) => left.id.localeCompare(right.id))
  const components = enchantments.length === 0
    ? ''
    : `[minecraft:enchantments={levels:{${enchantments.map(enchantment => `"${enchantment.id}":${enchantment.level}`).join(',')}}}]`
  return {
    kind: 'tool',
    username: options.username,
    slot,
    item,
    count,
    enchantments,
    command: `/item replace entity ${options.username ?? '@s'} ${slot} with ${item}${components} ${count}`
  }
}

export function createMobPlacement(options = {}) {
  const type = namespaced(options.type ?? 'zombie')
  const pos = normalizePos(options.pos ?? { x: 0, y: 64, z: 0 })
  const nbt = options.nbt ?? {}
  return {
    kind: 'mob',
    type,
    pos,
    nbt,
    command: `/summon ${type} ${pos.x} ${pos.y} ${pos.z}${nbtSuffix(nbt)}`
  }
}

export function createChestPlacement(options = {}) {
  const pos = normalizePos(options.pos ?? { x: 0, y: 64, z: 0 })
  const items = (options.items ?? []).map((item, index) => ({ ...normalizeSlot(item), slotIndex: item.slotIndex ?? index }))
  return {
    kind: 'chest',
    pos,
    items,
    commands: [
      `/setblock ${pos.x} ${pos.y} ${pos.z} minecraft:chest replace`,
      ...items.map(item =>
        `/item replace block ${pos.x} ${pos.y} ${pos.z} container.${item.slotIndex} with ${item.item} ${item.count}`
      )
    ]
  }
}

export function createVillagerOfferCapture(options = {}) {
  const pos = normalizePos(options.pos ?? { x: 0, y: 64, z: 0 })
  const profession = namespaced(options.profession ?? 'farmer')
  const offers = (options.offers ?? []).map((offer, index) => ({
    index,
    buy: normalizeStack(offer.buy ?? { item: 'minecraft:emerald', count: 1 }),
    buyB: offer.buyB ? normalizeStack(offer.buyB) : null,
    sell: normalizeStack(offer.sell ?? { item: 'minecraft:bread', count: 1 }),
    maxUses: offer.maxUses ?? 9999999
  }))
  return {
    kind: 'villagerOffers',
    pos,
    profession,
    offers,
    command: `/summon minecraft:villager ${pos.x} ${pos.y} ${pos.z}${villagerNbt(profession, offers)}`,
    capture: {
      windowType: 'minecraft:merchant',
      expectedOfferCount: offers.length
    }
  }
}

export function createFishingLoopControl(options = {}) {
  return {
    kind: 'fishingLoop',
    casts: options.casts ?? 3,
    timeoutMs: options.timeoutMs ?? 30_000,
    reelInOnBite: options.reelInOnBite ?? true,
    stopOnFirstCatch: options.stopOnFirstCatch ?? false,
    expectedLoot: (options.expectedLoot ?? []).map(normalizeStack)
  }
}

export function createItemEntityCollection(options = {}) {
  const pos = normalizePos(options.pos ?? { x: 0, y: 64, z: 0 })
  const stack = normalizeStack(options.stack ?? { item: 'minecraft:emerald', count: 1 })
  return {
    kind: 'itemEntityCollection',
    pos,
    stack,
    command: `/summon minecraft:item ${pos.x} ${pos.y} ${pos.z} {Item:{id:"${stack.item}",count:${stack.count}}}`,
    expectedInventoryDelta: { [stack.item]: stack.count }
  }
}

export function createPostReconnectSnapshot(session) {
  return {
    profile: session.profile,
    uuid: session.uuid,
    heldItem: summarizeItem(session.bot?.heldItem),
    inventory: summarizeInventory(session.bot?.inventory),
    position: summarizePosition(session.bot?.entity?.position),
    timeline: session.timeline.map(event => ({ name: event.name, summary: event.summary })),
    packetTrace: session.packetTrace.map(packet => ({ name: packet.name, state: packet.state, keys: packet.keys }))
  }
}

export function createLootEconomyFixture(options = {}) {
  const profile = options.profile ?? { username: 'VibeCraftBot' }
  return {
    name: options.name ?? 'loot-economy',
    profile,
    inventory: createInventorySetup(profile, options.inventory ?? []),
    tool: createToolEnchantmentSetup({ username: profile.username, ...(options.tool ?? {}) }),
    mob: createMobPlacement(options.mob ?? {}),
    chest: createChestPlacement(options.chest ?? {}),
    villager: createVillagerOfferCapture(options.villager ?? {}),
    fishing: createFishingLoopControl(options.fishing ?? {}),
    itemEntity: createItemEntityCollection(options.itemEntity ?? {})
  }
}

function normalizeSlot(slot) {
  return {
    slot: slot.slot ?? 'hotbar.0',
    ...normalizeStack(slot)
  }
}

function normalizeStack(stack) {
  return {
    item: namespaced(stack.item ?? stack.name ?? 'minecraft:air'),
    count: stack.count ?? 1
  }
}

function normalizePos(pos) {
  return {
    x: pos.x ?? 0,
    y: pos.y ?? 64,
    z: pos.z ?? 0
  }
}

function namespaced(id) {
  return id.includes(':') ? id : `minecraft:${id}`
}

function nbtSuffix(nbt) {
  const entries = Object.entries(nbt)
  if (entries.length === 0) return ''
  return ` ${JSON.stringify(Object.fromEntries(entries.sort(([left], [right]) => left.localeCompare(right))))}`
}

function villagerNbt(profession, offers) {
  const recipes = offers.map(offer => {
    const buyB = offer.buyB ? `,buyB:{id:"${offer.buyB.item}",count:${offer.buyB.count}}` : ''
    return `{buy:{id:"${offer.buy.item}",count:${offer.buy.count}}${buyB},sell:{id:"${offer.sell.item}",count:${offer.sell.count}},maxUses:${offer.maxUses}}`
  }).join(',')
  return ` {VillagerData:{profession:"${profession}",level:5,type:"minecraft:plains"},Offers:{Recipes:[${recipes}]}}`
}

function summarizeItem(item) {
  if (!item) return null
  return {
    name: item.name ?? null,
    count: item.count ?? 1,
    displayName: item.displayName ?? null
  }
}

function summarizeInventory(inventory) {
  if (!inventory?.slots) return []
  return inventory.slots
    .map((item, index) => item ? { slot: index, ...summarizeItem(item) } : null)
    .filter(Boolean)
}

function summarizePosition(position) {
  if (!position) return null
  return { x: position.x, y: position.y, z: position.z }
}
