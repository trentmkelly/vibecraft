// Mineflayer block interaction parity tests: right-click use, sneak-use bypass,
// client prediction rollback, block entity update tags after right-click,
// neighbor-shape updates after offline-mode placement.

export function createBlockInteractionParityPlan(options = {}) {
  return {
    name: 'mineflayer-block-interaction-parity',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'BlockInteractBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      'spawn-protection': '0'
    },
    steps: [
      'right-click-block-use',
      'sneak-use-bypass',
      'client-prediction-rollback',
      'block-entity-tag-after-use',
      'neighbor-shape-update-after-placement'
    ]
  }
}

export function summarizeBlockInteractionParity(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => ['block', 'interaction'].includes(e.name))
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
    case 'right-click-block-use': return actions.includes('interaction.right_click_use')
    case 'sneak-use-bypass': return actions.includes('interaction.sneak_use_bypass')
    case 'client-prediction-rollback': return actions.includes('interaction.prediction_rollback')
    case 'block-entity-tag-after-use': return actions.includes('interaction.block_entity_tag')
    case 'neighbor-shape-update-after-placement': return actions.includes('interaction.neighbor_shape_update')
    default: return false
  }
}

export function recordInteractionEvent(session, action, details = {}) {
  session.timeline.push({ name: 'interaction', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

export function planRightClickUse(blockId, expectedResult) {
  return {
    action: 'interaction.right_click_use',
    blockId,
    expectedResult: expectedResult ?? 'SUCCESS'
  }
}

export function planSneakUseBypass(blockId, heldItemId) {
  return {
    action: 'interaction.sneak_use_bypass',
    blockId,
    heldItemId,
    description: 'sneaking + right-click with item skips block use handler, routes to item use'
  }
}

export function planPredictionRollback(targetPos, placedBlockId) {
  return {
    action: 'interaction.prediction_rollback',
    targetPos,
    placedBlockId,
    description: 'server rejects placement → client receives corrective block update packet'
  }
}

export function planBlockEntityTagAfterUse(blockId, expectedNbtKeys) {
  return {
    action: 'interaction.block_entity_tag',
    blockId,
    expectedNbtKeys: expectedNbtKeys ?? []
  }
}

export function planNeighborShapeUpdate(placedBlockId, expectedNeighborUpdate) {
  return {
    action: 'interaction.neighbor_shape_update',
    placedBlockId,
    expectedNeighborUpdate
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('block_interaction_parity scenarios defined — run via the test harness')
}
