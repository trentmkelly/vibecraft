import { once } from 'node:events'

export async function digBlock(session, target, options = {}) {
  const block = options.block ?? session.bot.blockAt(target)
  if (!block) throw new Error(`No block to dig at ${formatPos(target)}`)
  const before = summarizeBlock(block)
  const result = await session.bot.dig(block, options.forceLook ?? true)
  return recordBlockEvent(session, 'block.dig', {
    before,
    result: summarizeResult(result)
  })
}

export async function placeBlock(session, target, options = {}) {
  const referenceBlock = options.block ?? session.bot.blockAt(target)
  if (!referenceBlock) throw new Error(`No reference block to place against at ${formatPos(target)}`)
  const faceVector = options.faceVector ?? { x: 0, y: 1, z: 0 }
  const result = await session.bot.placeBlock(referenceBlock, faceVector)
  return recordBlockEvent(session, 'block.place', {
    reference: summarizeBlock(referenceBlock),
    faceVector,
    result: summarizeResult(result)
  })
}

export async function useBlock(session, target, options = {}) {
  const block = options.block ?? session.bot.blockAt(target)
  if (!block) throw new Error(`No block to use at ${formatPos(target)}`)
  const result = await (session.bot.activateBlock
    ? session.bot.activateBlock(block, options.direction)
    : session.bot.openBlock(block))
  return recordBlockEvent(session, 'block.use', {
    block: summarizeBlock(block),
    direction: options.direction ?? null,
    result: summarizeResult(result)
  })
}

export async function expectDeniedInteraction(session, action, options = {}) {
  const before = summarizeBlock(options.blockBefore ?? session.bot.blockAt(options.target))
  let error = null
  try {
    await action()
  } catch (err) {
    error = err
  }
  const after = summarizeBlock(options.blockAfter ?? session.bot.blockAt(options.target))
  const ok = Boolean(error) || sameBlock(before, after)
  return recordBlockEvent(session, 'block.denied', {
    ok,
    reason: options.reason ?? normalizeError(error),
    before,
    after
  })
}

export function assertSpawnProtection(session, attempted = {}) {
  // DedicatedServer.isUnderSpawnProtection (26.1.2): Chebyshev distance
  // max(|dx|, |dz|) from the respawn pos compared against the radius, NOT a
  // Euclidean distance. Any op (op-list membership, i.e. opLevel >= 1) bypasses,
  // and a radius <= 0 disables protection entirely.
  const distance = chebyshevDistance(attempted.position ?? { x: 0, z: 0 }, attempted.spawn ?? { x: 0, z: 0 })
  const protectedRadius = attempted.protectedRadius ?? 16
  const opLevel = attempted.opLevel ?? 0
  const expectedDenied = protectedRadius > 0 && distance <= protectedRadius && opLevel < 1
  return recordBlockEvent(session, 'block.spawn_protection', {
    ok: expectedDenied === Boolean(attempted.denied),
    expectedDenied,
    denied: Boolean(attempted.denied),
    distance,
    protectedRadius,
    opLevel
  })
}

export async function waitForBlockUpdate(session, expected = {}, options = {}) {
  const event = await onceWithTimeout(session.bot, 'blockUpdate', options.timeoutMs ?? 30_000)
  const oldBlock = summarizeBlock(event[0])
  const newBlock = summarizeBlock(event[1])
  const ok = blockMatches(newBlock, expected)
  return recordBlockEvent(session, 'block.update_visible', {
    ok,
    expected,
    oldBlock,
    newBlock
  })
}

export function createBlockInteractionScenarioPlan(options = {}) {
  return {
    name: options.name ?? 'mineflayer-block-interactions',
    mode: 'offline',
    auth: 'offline',
    steps: [
      { action: 'block.dig', target: options.digTarget ?? { x: 0, y: 64, z: 1 } },
      {
        action: 'block.place',
        target: options.placeTarget ?? { x: 0, y: 64, z: 1 },
        faceVector: options.faceVector ?? { x: 0, y: 1, z: 0 }
      },
      { action: 'block.use', target: options.useTarget ?? { x: 1, y: 64, z: 0 } },
      {
        action: 'block.denied',
        reason: options.deniedReason ?? 'server rejected interaction'
      },
      {
        action: 'block.spawn_protection',
        protectedRadius: options.protectedRadius ?? 16
      },
      {
        action: 'block.update_visible',
        expected: options.expectedUpdate ?? { name: 'stone' }
      }
    ]
  }
}

function recordBlockEvent(session, action, details = {}) {
  const event = {
    name: 'block',
    at: Date.now(),
    summary: [action, details]
  }
  session.timeline.push(event)
  return { action, ...details }
}

function summarizeBlock(block) {
  if (!block) return null
  return {
    name: block.name ?? null,
    type: block.type ?? null,
    position: block.position ? { x: block.position.x, y: block.position.y, z: block.position.z } : null,
    boundingBox: block.boundingBox ?? null,
    metadata: block.metadata ?? null
  }
}

function sameBlock(left, right) {
  return JSON.stringify(left) === JSON.stringify(right)
}

function blockMatches(actual, expected = {}) {
  if (!actual) return false
  if (expected.name !== undefined && actual.name !== expected.name) return false
  if (expected.type !== undefined && actual.type !== expected.type) return false
  if (expected.position !== undefined && !samePos(actual.position, expected.position)) return false
  return true
}

function samePos(left, right) {
  return left?.x === right?.x && left?.y === right?.y && left?.z === right?.z
}

function chebyshevDistance(left, right) {
  const dx = Math.abs((left.x ?? 0) - (right.x ?? 0))
  const dz = Math.abs((left.z ?? 0) - (right.z ?? 0))
  return Math.max(dx, dz)
}

// Block reach enforcement (Player.canInteractWithBlock, 26.1.2):
//   maxRange = blockInteractionRange() + buffer ; reachable iff
//   new AABB(pos).distanceToSqr(eyePos) < maxRange*maxRange
// DEFAULT_BLOCK_INTERACTION_RANGE = 4.5 (BLOCK_INTERACTION_RANGE attribute base),
// and the server packet handler validates with a 1.0 padding -> effective 5.5.
// Creative does NOT extend block reach (only entity-attack reach differs); both
// modes use the same 4.5 attribute server-side. The item's "5.0" is outdated.
export const DEFAULT_BLOCK_INTERACTION_RANGE = 4.5
export const SERVER_REACH_BUFFER = 1.0

// AABB.distanceToSqr for a unit block [pos, pos+1]: per-axis clamp of the eye
// point outside the box, summed squares (mirrors block_aabb_distance_sq).
export function blockAabbDistanceSq(blockPos, eyePos) {
  const axis = (p, e) => Math.max(p - e, e - (p + 1), 0)
  const dx = axis(blockPos.x ?? 0, eyePos.x ?? 0)
  const dy = axis(blockPos.y ?? 0, eyePos.y ?? 0)
  const dz = axis(blockPos.z ?? 0, eyePos.z ?? 0)
  return dx * dx + dy * dy + dz * dz
}

export function isWithinBlockReach(eyePos, blockPos, interactionRange = DEFAULT_BLOCK_INTERACTION_RANGE) {
  const maxRange = interactionRange + SERVER_REACH_BUFFER
  return blockAabbDistanceSq(blockPos, eyePos) < maxRange * maxRange
}

function normalizeError(error) {
  if (!error) return null
  return error.code ?? error.message ?? String(error)
}

function summarizeResult(result) {
  if (result == null) return null
  if (typeof result === 'string' || typeof result === 'number' || typeof result === 'boolean') return result
  return result.toString?.() ?? Object.prototype.toString.call(result)
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
