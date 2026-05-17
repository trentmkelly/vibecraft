import { once } from 'node:events'

export async function waitForEntitySpawn(session, expected = {}, options = {}) {
  const [entity] = await onceWithTimeout(session.bot, 'entitySpawn', options.timeoutMs ?? 30_000)
  const observed = summarizeEntity(entity)
  return recordEntityEvent(session, 'entity.spawn', {
    ok: entityMatches(observed, expected),
    expected,
    observed
  })
}

export async function waitForEntityDespawn(session, expected = {}, options = {}) {
  const [entity] = await onceWithTimeout(session.bot, 'entityGone', options.timeoutMs ?? 30_000)
  const observed = summarizeEntity(entity)
  return recordEntityEvent(session, 'entity.despawn', {
    ok: entityMatches(observed, expected),
    expected,
    observed
  })
}

export async function waitForMetadataUpdate(session, expected = {}, options = {}) {
  const [entity, metadata] = await onceWithTimeout(session.bot, 'entityUpdate', options.timeoutMs ?? 30_000)
  const observed = {
    entity: summarizeEntity(entity),
    metadata: summarizeMetadata(metadata ?? entity?.metadata)
  }
  return recordEntityEvent(session, 'entity.metadata', {
    ok: entityMatches(observed.entity, expected.entity ?? {}) && metadataMatches(observed.metadata, expected.metadata ?? {}),
    expected,
    observed
  })
}

export async function waitForDamageAnimation(session, expected = {}, options = {}) {
  const [entity] = await onceWithTimeout(session.bot, 'entityHurt', options.timeoutMs ?? 30_000)
  const observed = summarizeEntity(entity)
  return recordEntityEvent(session, 'entity.damage_animation', {
    ok: entityMatches(observed, expected),
    expected,
    observed
  })
}

export async function waitForItemPickup(session, expected = {}, options = {}) {
  const [collector, collected] = await onceWithTimeout(session.bot, 'playerCollect', options.timeoutMs ?? 30_000)
  const observed = {
    collector: summarizeEntity(collector),
    collected: summarizeEntity(collected)
  }
  return recordEntityEvent(session, 'entity.item_pickup', {
    ok: entityMatches(observed.collector, expected.collector ?? {}) && entityMatches(observed.collected, expected.collected ?? {}),
    expected,
    observed
  })
}

export async function attackEntity(session, target, options = {}) {
  const entity = options.entity ?? resolveEntity(session, target)
  if (!entity) throw new Error(`No entity to attack: ${target}`)
  const result = await session.bot.attack(entity, options.swing ?? true)
  return recordEntityEvent(session, 'entity.combat', {
    target: summarizeEntity(entity),
    swing: options.swing ?? true,
    result: summarizeResult(result)
  })
}

export function createEntityObservationScenarioPlan(options = {}) {
  return {
    name: options.name ?? 'mineflayer-entity-observations',
    mode: 'offline',
    auth: 'offline',
    steps: [
      { action: 'entity.spawn', expected: options.spawn ?? { name: 'zombie' } },
      { action: 'entity.metadata', expected: options.metadata ?? { entity: { name: 'zombie' } } },
      { action: 'entity.damage_animation', expected: options.damage ?? { name: 'zombie' } },
      { action: 'entity.item_pickup', expected: options.pickup ?? { collected: { name: 'item' } } },
      { action: 'entity.combat', target: options.combatTarget ?? 'zombie' },
      { action: 'entity.despawn', expected: options.despawn ?? { name: 'zombie' } }
    ]
  }
}

function resolveEntity(session, target) {
  if (typeof target === 'object') return target
  return Object.values(session.bot.entities ?? {}).find(entity =>
    entity.name === target || entity.username === target || entity.displayName === target
  )
}

function recordEntityEvent(session, action, details = {}) {
  const event = {
    name: 'entity',
    at: Date.now(),
    summary: [action, details]
  }
  session.timeline.push(event)
  return { action, ...details }
}

function summarizeEntity(entity) {
  if (!entity) return null
  return {
    id: entity.id ?? null,
    name: entity.name ?? entity.username ?? entity.displayName ?? null,
    type: entity.type ?? null,
    kind: entity.kind ?? entity.objectType ?? null,
    metadata: summarizeMetadata(entity.metadata),
    position: entity.position ? { x: entity.position.x, y: entity.position.y, z: entity.position.z } : null,
    health: entity.health ?? null
  }
}

function summarizeMetadata(metadata) {
  if (!metadata) return {}
  if (Array.isArray(metadata)) {
    return Object.fromEntries(metadata.map((entry, index) => [index, entry?.value ?? entry]))
  }
  return { ...metadata }
}

function entityMatches(actual, expected = {}) {
  if (!actual) return false
  if (expected.id !== undefined && actual.id !== expected.id) return false
  if (expected.name !== undefined && actual.name !== expected.name) return false
  if (expected.type !== undefined && actual.type !== expected.type) return false
  if (expected.health !== undefined && actual.health !== expected.health) return false
  return true
}

function metadataMatches(actual, expected = {}) {
  return Object.entries(expected).every(([key, value]) => actual[key] === value)
}

function summarizeResult(result) {
  if (result == null) return null
  if (typeof result === 'string' || typeof result === 'number' || typeof result === 'boolean') return result
  return result.toString?.() ?? Object.prototype.toString.call(result)
}

function onceWithTimeout(emitter, event, timeoutMs) {
  return Promise.race([
    once(emitter, event),
    new Promise((_, reject) => {
      setTimeout(() => reject(new Error(`Timed out waiting for ${event}`)), timeoutMs)
    })
  ])
}
