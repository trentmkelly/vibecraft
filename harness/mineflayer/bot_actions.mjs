import { once } from 'node:events'
import { connectObservedOfflineBot } from './login_session.mjs'

export async function waitForSpawn(session, options = {}) {
  if (session.timeline.some(event => event.name === 'spawn')) {
    return recordAction(session, 'waitForSpawn', { alreadySpawned: true })
  }
  await onceWithTimeout(session.bot, 'spawn', options.timeoutMs ?? 30_000)
  return recordAction(session, 'waitForSpawn', { alreadySpawned: false })
}

export function assertHeldItem(session, expected = {}) {
  const actual = summarizeItem(session.bot?.heldItem)
  const mismatches = []
  if (expected.name !== undefined && actual.name !== expected.name) mismatches.push(['name', expected.name, actual.name])
  if (expected.count !== undefined && actual.count !== expected.count) mismatches.push(['count', expected.count, actual.count])
  if (expected.displayName !== undefined && actual.displayName !== expected.displayName) {
    mismatches.push(['displayName', expected.displayName, actual.displayName])
  }
  const result = mismatches.length === 0
    ? { ok: true, actual }
    : { ok: false, code: 'held_item.mismatch', expected, actual, mismatches }
  recordAction(session, 'assertHeldItem', result)
  return result
}

export function issueCommand(session, command) {
  const normalized = command.startsWith('/') ? command : `/${command}`
  session.bot.chat(normalized)
  return recordAction(session, 'issueCommand', { command: normalized })
}

export async function placeBlock(session, target, options = {}) {
  const referenceBlock = options.block ?? session.bot.blockAt(target)
  if (!referenceBlock) throw new Error(`No reference block at ${formatPos(target)}`)
  const faceVector = options.faceVector ?? { x: 0, y: 1, z: 0 }
  const result = await session.bot.placeBlock(referenceBlock, faceVector)
  return recordAction(session, 'placeBlock', {
    position: blockPosition(referenceBlock),
    faceVector,
    result: summarizeActionResult(result)
  })
}

export async function breakBlock(session, target, options = {}) {
  const block = options.block ?? session.bot.blockAt(target)
  if (!block) throw new Error(`No block to break at ${formatPos(target)}`)
  const result = await session.bot.dig(block, options.forceLook ?? true)
  return recordAction(session, 'breakBlock', {
    position: blockPosition(block),
    result: summarizeActionResult(result)
  })
}

export async function openWindow(session, target, options = {}) {
  const block = options.block ?? session.bot.blockAt(target)
  if (!block) throw new Error(`No block to open at ${formatPos(target)}`)
  const window = await session.bot.openBlock(block)
  const summary = {
    position: blockPosition(block),
    window: summarizeWindow(window)
  }
  recordAction(session, 'openWindow', summary)
  return window
}

export async function forceReconnect(session, options = {}) {
  session.bot?.end()
  recordAction(session, 'forceReconnect.end', { username: session.profile.username })
  const observed = await (options.connectBot ?? connectObservedOfflineBot)({
    host: options.host ?? session.endpoint?.host ?? '127.0.0.1',
    port: options.port ?? session.endpoint?.port,
    version: options.version ?? session.endpoint?.version,
    username: options.username ?? session.profile.username,
    timeoutMs: options.timeoutMs ?? 30_000,
    timeline: session.timeline,
    packetTrace: session.packetTrace
  })
  session.bot = observed.bot
  session.profile = observed.profile
  session.uuid = observed.profile.actualUuid ?? observed.profile.expectedUuid
  return recordAction(session, 'forceReconnect.connected', {
    username: session.profile.username,
    uuid: session.uuid
  })
}

export function captureVanillaComparisonTraces(officialSession, rustCraftSession) {
  return {
    official: traceSummary(officialSession),
    rustCraft: traceSummary(rustCraftSession)
  }
}

function recordAction(session, action, details = {}) {
  const event = {
    name: 'action',
    at: Date.now(),
    summary: [action, details]
  }
  session.timeline.push(event)
  return { action, ...details }
}

function traceSummary(session) {
  return {
    profile: session.profile,
    uuid: session.uuid,
    timeline: session.timeline.map(event => ({
      name: event.name,
      summary: event.summary
    })),
    packetTrace: session.packetTrace.map(packet => ({
      name: packet.name,
      state: packet.state,
      keys: packet.keys
    }))
  }
}

function summarizeItem(item) {
  if (!item) return { name: null, count: 0, displayName: null }
  return {
    name: item.name ?? item.type ?? null,
    count: item.count ?? 1,
    displayName: item.displayName ?? null
  }
}

function summarizeWindow(window) {
  if (!window) return null
  return {
    id: window.id,
    type: window.type,
    title: typeof window.title === 'string' ? window.title : window.title?.toString?.()
  }
}

function blockPosition(block) {
  return block.position ?? block
}

function formatPos(pos) {
  if (!pos) return '<missing>'
  return `${pos.x},${pos.y},${pos.z}`
}

function summarizeActionResult(result) {
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
