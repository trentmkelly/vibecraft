export const ENTITY_BEHAVIOR_STEPS = [
  'spawning-finalize-spawn',
  'ai-goal-tick',
  'pathfinding-navigation-tick',
  'combat-damage-animation',
  'drops-loot-table',
  'save-load-roundtrip',
  'network-metadata-sync'
]

export const ENTITY_BEHAVIOR_JAVA_ORACLES = {
  'spawning-finalize-spawn': 'net/minecraft/world/entity/Mob.java#finalizeSpawn',
  'ai-goal-tick': 'net/minecraft/world/entity/Mob.java#serverAiStep',
  'pathfinding-navigation-tick': 'net/minecraft/world/entity/ai/navigation/PathNavigation.java#tick',
  'combat-damage-animation': 'net/minecraft/world/entity/LivingEntity.java#hurt/dropFromLootTable',
  'drops-loot-table': 'net/minecraft/world/entity/LivingEntity.java#dropFromLootTable',
  'save-load-roundtrip': 'net/minecraft/world/entity/Entity.java#save/load',
  'network-metadata-sync': 'net/minecraft/world/entity/Entity.java#SynchedEntityData'
}

export function createEntityBehaviorParityPlan(options = {}) {
  return {
    name: options.name ?? 'mineflayer-entity-behavior-parity',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'EntityBehaviorBot',
    comparedAgainst: 'official-server.jar',
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      gamemode: 'survival',
      difficulty: options.difficulty ?? 'normal',
      'spawn-protection': '0'
    },
    steps: ENTITY_BEHAVIOR_STEPS.map(step => ({
      id: step,
      oracle: ENTITY_BEHAVIOR_JAVA_ORACLES[step],
      requiredEvidence: requiredEvidenceForStep(step)
    }))
  }
}

export async function runEntityBehaviorParityScenario(options = {}) {
  const plan = createEntityBehaviorParityPlan(options)
  const evidence = await (options.probe ?? missingProbe)(plan, options)
  return {
    plan,
    evidence,
    summary: summarizeEntityBehaviorParity(evidence, plan)
  }
}

export function summarizeEntityBehaviorParity(evidence, plan = createEntityBehaviorParityPlan()) {
  const events = (evidence.timeline ?? []).filter(event => event.name === 'entity_behavior')
  const steps = Object.fromEntries(plan.steps.map(step => {
    const event = events.find(candidate => candidate.summary?.[0] === step.id)
    return [step.id, evidenceSatisfiesStep(event?.summary?.[1], step)]
  }))
  return {
    ok: Object.values(steps).every(Boolean),
    comparedAgainst: evidence.comparedAgainst ?? null,
    steps
  }
}

export function recordEntityBehaviorEvidence(evidence, stepId, details = {}) {
  evidence.timeline ??= []
  evidence.timeline.push({
    name: 'entity_behavior',
    at: Date.now(),
    summary: [stepId, details]
  })
  return { stepId, ...details }
}

export function requiredEvidenceForStep(step) {
  switch (step) {
    case 'spawning-finalize-spawn':
      return ['spawnPacket', 'finalizeSpawnState', 'vanillaDiff']
    case 'ai-goal-tick':
      return ['goalTickObserved', 'targetSelection', 'vanillaDiff']
    case 'pathfinding-navigation-tick':
      return ['pathNodes', 'movementPackets', 'vanillaDiff']
    case 'combat-damage-animation':
      return ['attackPacket', 'hurtAnimation', 'healthDelta', 'vanillaDiff']
    case 'drops-loot-table':
      return ['deathEvent', 'itemEntities', 'xp', 'vanillaDiff']
    case 'save-load-roundtrip':
      return ['preSaveNbt', 'postLoadNbt', 'reconnectObserved', 'vanillaDiff']
    case 'network-metadata-sync':
      return ['metadataPacket', 'fieldValues', 'vanillaDiff']
    default:
      return []
  }
}

function evidenceSatisfiesStep(details, step) {
  if (!details) return false
  if (details.comparedToVanilla !== true) return false
  return step.requiredEvidence.every(key => Boolean(details[key]))
}

async function missingProbe() {
  throw new Error('entity behavior parity probe requires live VibeCraft and official-server.jar fixtures')
}
