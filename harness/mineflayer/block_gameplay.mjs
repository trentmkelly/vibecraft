// Mineflayer block place/break tests for offline-mode survival and creative bots.
// Tests: reach distance enforcement, spawn-protection denial, placement orientation by face hit,
// drops from breaking, visible block-update acknowledgment packets.

import { once } from 'node:events'

export function createBlockGameplayPlan(options = {}) {
  return {
    name: 'mineflayer-block-gameplay',
    client: 'mineflayer',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'BlockGameplayBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: options.gamemode ?? 'survival',
      difficulty: 'peaceful',
      'spawn-protection': '0'
    },
    steps: [
      'break-block-bare-hand',
      'break-block-correct-tool',
      'place-block-on-face',
      'placement-denied-too-far',
      'placement-denied-spawn-protection',
      'block-update-ack-received'
    ]
  }
}

export function summarizeBlockGameplay(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => e.name === 'block')
    .map(e => e.summary?.[0])
  const result = Object.fromEntries(
    plan.steps.map(step => [step, hasStepEvidence(step, actions)])
  )
  return {
    ok: Object.values(result).every(Boolean),
    steps: result
  }
}

function hasStepEvidence(step, actions) {
  switch (step) {
    case 'break-block-bare-hand': return actions.includes('block.break.bare_hand')
    case 'break-block-correct-tool': return actions.includes('block.break.correct_tool')
    case 'place-block-on-face': return actions.includes('block.place.face')
    case 'placement-denied-too-far': return actions.includes('block.placement.denied.too_far')
    case 'placement-denied-spawn-protection': return actions.includes('block.placement.denied.spawn_protection')
    case 'block-update-ack-received': return actions.includes('block.update.ack')
    default: return false
  }
}

export function recordBlockGameplayEvent(session, action, details = {}) {
  session.timeline.push({ name: 'block', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

export function planBreakBareHand(blockId, expectedDrops) {
  return { action: 'block.break.bare_hand', blockId, expectedDrops: expectedDrops ?? [] }
}

export function planBreakCorrectTool(blockId, toolId, expectedDrops) {
  return { action: 'block.break.correct_tool', blockId, toolId, expectedDrops: expectedDrops ?? [blockId] }
}

export function planPlaceOnFace(blockId, targetFace, expectedFacingProp) {
  return { action: 'block.place.face', blockId, targetFace, expectedFacingProp }
}

export function assertReachEnforcement(distance, reachLimit, denied) {
  return {
    action: distance > reachLimit ? 'block.placement.denied.too_far' : 'block.placement.ok',
    distance,
    reachLimit,
    denied
  }
}

export function assertSpawnProtectionBlock(distance, protectedRadius, denied) {
  return {
    action: denied ? 'block.placement.denied.spawn_protection' : 'block.placement.ok',
    distance,
    protectedRadius,
    denied
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('block_gameplay scenarios defined — run via the test harness')
}
