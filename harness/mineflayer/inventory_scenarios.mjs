import { once } from 'node:events'

export async function selectHotbarSlot(session, slot) {
  assertHotbarSlot(slot)
  session.bot.setQuickBarSlot(slot)
  return recordInventoryEvent(session, 'hotbar.select', {
    slot,
    heldItem: summarizeItem(session.bot.heldItem)
  })
}

export async function waitForItemPickup(session, expected = {}, options = {}) {
  const event = await onceWithTimeout(session.bot, 'playerCollect', options.timeoutMs ?? 30_000)
  const summary = {
    collector: summarizeEntity(event[0]),
    collected: summarizeEntity(event[1]),
    inventory: summarizeInventory(session.bot.inventory)
  }
  return recordInventoryEvent(session, 'item.pickup', {
    ...summary,
    assertion: assertInventoryContains(session, expected)
  })
}

export async function dropHeldItem(session, options = {}) {
  const stack = options.stack ?? session.bot.heldItem
  if (!stack) throw new Error('Cannot drop held item: no held item is present')
  const dropped = options.fullStack === false && session.bot.toss
    ? await session.bot.toss(stack.type ?? stack.name, null, options.count ?? 1)
    : await session.bot.tossStack(stack)
  return recordInventoryEvent(session, 'item.drop', {
    fullStack: options.fullStack !== false,
    requested: summarizeItem(stack),
    result: summarizeResult(dropped),
    heldItem: summarizeItem(session.bot.heldItem)
  })
}

export async function openAndCloseWindow(session, target, options = {}) {
  const block = options.block ?? session.bot.blockAt(target)
  if (!block) throw new Error(`No block to open at ${formatPos(target)}`)
  const window = await session.bot.openBlock(block)
  const opened = summarizeWindow(window)
  window.close?.()
  return recordInventoryEvent(session, 'window.open_close', {
    block: block.position ?? target,
    opened,
    closed: true
  })
}

export async function clickWindowSlot(session, window, slot, mouseButton = 0, mode = 0) {
  const before = summarizeWindowSlot(window, slot)
  const result = await session.bot.clickWindow(slot, mouseButton, mode)
  const after = summarizeWindowSlot(window, slot)
  return recordInventoryEvent(session, 'window.click_slot', {
    window: summarizeWindow(window),
    slot,
    mouseButton,
    mode,
    before,
    after,
    result: summarizeResult(result)
  })
}

export function assertHeldItemSync(session, expected = {}) {
  const actual = summarizeItem(session.bot.heldItem)
  const selectedSlot = session.bot.quickBarSlot ?? session.bot.heldItemSlot ?? null
  const inventoryItem = selectedSlot == null
    ? null
    : summarizeItem(session.bot.inventory?.slots?.[36 + selectedSlot])
  const ok = itemMatches(actual, expected) && itemMatches(inventoryItem, expected)
  return recordInventoryEvent(session, 'held_item.sync', {
    ok,
    selectedSlot,
    expected,
    actual,
    inventoryItem
  })
}

export function assertCarriedItemCorrection(session, expected = {}) {
  const carried = summarizeItem(session.bot.inventory?.selectedItem ?? session.bot.inventory?.cursor)
  const serverSlot = expected.slot == null
    ? null
    : summarizeItem(session.bot.inventory?.slots?.[expected.slot])
  const ok = itemMatches(carried, expected.carried ?? {}) && itemMatches(serverSlot, expected.serverSlot ?? {})
  return recordInventoryEvent(session, 'carried_item.correction', {
    ok,
    expected,
    carried,
    serverSlot
  })
}

export function createInventoryScenarioPlan(options = {}) {
  return {
    name: options.name ?? 'mineflayer-inventory',
    steps: [
      { action: 'hotbar.select', slot: options.hotbarSlot ?? 0 },
      { action: 'item.pickup', expected: options.pickup ?? { name: 'stick', count: 1 } },
      { action: 'item.drop', fullStack: options.fullStackDrop ?? true },
      { action: 'window.open_close', target: options.windowTarget ?? { x: 0, y: 64, z: 0 } },
      { action: 'window.click_slot', slot: options.windowSlot ?? 0, mouseButton: 0, mode: 0 },
      { action: 'held_item.sync', expected: options.heldItem ?? { name: 'stick', count: 1 } },
      {
        action: 'carried_item.correction',
        expected: options.carriedCorrection ?? { carried: { name: null, count: 0 } }
      }
    ]
  }
}

function recordInventoryEvent(session, action, details = {}) {
  const event = {
    name: 'inventory',
    at: Date.now(),
    summary: [action, details]
  }
  session.timeline.push(event)
  return { action, ...details }
}

function assertInventoryContains(session, expected = {}) {
  const inventory = summarizeInventory(session.bot.inventory)
  const matches = inventory.filter(item => itemMatches(item, expected))
  const count = matches.reduce((total, item) => total + item.count, 0)
  const requiredCount = expected.count ?? 1
  return {
    ok: count >= requiredCount,
    expected,
    count,
    matches
  }
}

function summarizeInventory(inventory) {
  if (!inventory?.slots) return []
  return inventory.slots
    .map((item, slot) => item ? { slot, ...summarizeItem(item) } : null)
    .filter(Boolean)
}

function summarizeWindow(window) {
  if (!window) return null
  return {
    id: window.id,
    type: window.type,
    title: typeof window.title === 'string' ? window.title : window.title?.toString?.() ?? null
  }
}

function summarizeWindowSlot(window, slot) {
  return summarizeItem(window?.slots?.[slot])
}

function summarizeEntity(entity) {
  if (!entity) return null
  return {
    id: entity.id ?? null,
    name: entity.name ?? entity.username ?? entity.displayName ?? null,
    position: entity.position ? { x: entity.position.x, y: entity.position.y, z: entity.position.z } : null
  }
}

function summarizeItem(item) {
  if (!item) return { name: null, count: 0, displayName: null, type: null }
  return {
    name: item.name ?? null,
    count: item.count ?? 1,
    displayName: item.displayName ?? null,
    type: item.type ?? null
  }
}

function itemMatches(actual, expected = {}) {
  if (!actual) return false
  if (expected.name !== undefined && actual.name !== expected.name) return false
  if (expected.count !== undefined && actual.count !== expected.count) return false
  if (expected.displayName !== undefined && actual.displayName !== expected.displayName) return false
  return true
}

function summarizeResult(result) {
  if (result == null) return null
  if (typeof result === 'string' || typeof result === 'number' || typeof result === 'boolean') return result
  return result.toString?.() ?? Object.prototype.toString.call(result)
}

function assertHotbarSlot(slot) {
  if (!Number.isInteger(slot) || slot < 0 || slot > 8) {
    throw new RangeError(`Hotbar slot must be an integer from 0 to 8, got ${slot}`)
  }
}

function formatPos(pos) {
  if (!pos) return '<missing>'
  return `${pos.x},${pos.y},${pos.z}`
}

function onceWithTimeout(emitter, event, timeoutMs) {
  return Promise.race([
    once(emitter, event),
    new Promise((_, reject) => {
      setTimeout(() => reject(new Error(`Timed out waiting for ${event}`)), timeoutMs)
    })
  ])
}
