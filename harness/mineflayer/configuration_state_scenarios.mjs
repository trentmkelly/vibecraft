import { configurationCompletionManifest } from './configuration_completion_manifest.mjs'

export function createConfigurationStatePlan() {
  return {
    name: 'mineflayer-offline-configuration-state',
    mode: 'offline',
    auth: 'offline',
    client: 'raw-26.1.2-probe-until-mineflayer-supports-protocol-775',
    required: [
      'enabled-features-before-finish',
      'registry-order-before-finish',
      'tag-packets-before-finish',
      'known-packs-before-finish',
      'finish-configuration-after-known-packs',
      'play-entry-after-client-finish'
    ],
    manifest: configurationCompletionManifest
  }
}

export function createConfigurationCustomPayloadPlan() {
  return {
    name: 'mineflayer-offline-configuration-custom-payload',
    mode: 'offline',
    auth: 'offline',
    required: [
      'records-unknown-custom-payload',
      'records-brand-exchange',
      'records-client-information',
      'records-cookie-request-response',
      'records-configuration-disconnect'
    ]
  }
}

export function createConfigurationReplayPlan() {
  return {
    name: 'mineflayer-offline-configuration-replay',
    mode: 'offline',
    auth: 'offline',
    required: [
      'records-official-configuration-transcript',
      'records-rustcraft-configuration-transcript',
      'matches-login-milestones',
      'matches-configuration-milestones',
      'matches-play-entry-milestones',
      'no-hidden-sleeps',
      'no-retry-only-success'
    ]
  }
}

export function summarizeConfigurationStateEvidence(evidence, plan = createConfigurationStatePlan()) {
  const configIds = evidence.configPackets?.map(packet => packet.id) ?? []
  const registryOrder = evidence.configPackets
    ?.filter(packet => packet.id === 7)
    .map(packet => packet.registry) ?? []
  const tagsPacket = evidence.configPackets?.find(packet => packet.id === 13)
  const knownPacksPacket = evidence.configPackets?.find(packet => packet.id === 14)
  const finishIndex = configIds.lastIndexOf(3)
  const knownPackIndex = configIds.indexOf(14)
  const playIds = evidence.playPackets?.map(packet => packet.id) ?? []

  const checks = {
    'enabled-features-before-finish': configIds.indexOf(12) >= 0 && configIds.indexOf(12) < finishIndex,
    'registry-order-before-finish': sameArray(registryOrder, plan.manifest.registryOrder),
    'tag-packets-before-finish': Boolean(tagsPacket) && configIds.indexOf(13) < finishIndex,
    'known-packs-before-finish': Boolean(knownPacksPacket) && knownPackIndex < finishIndex && hasManifestKnownPack(knownPacksPacket, plan.manifest),
    'finish-configuration-after-known-packs': finishIndex > knownPackIndex,
    'play-entry-after-client-finish': [49, 10, 64, 105, 72, 43, 97, 94, 95, 38, 12, 45, 11].every(id => playIds.includes(id))
  }

  const missing = plan.required.filter(requirement => !checks[requirement])
  return {
    ok: missing.length === 0,
    checks,
    missing
  }
}

export function summarizeConfigurationCustomPayloadEvidence(evidence, plan = createConfigurationCustomPayloadPlan()) {
  const packets = evidence.packetTrace ?? []
  const events = evidence.timeline ?? []
  const packetNames = packets.map(packet => packet.name)
  const payloadChannels = packets.map(packet => packet.channel).filter(Boolean)

  const checks = {
    'records-unknown-custom-payload': payloadChannels.some(channel => channel !== 'minecraft:brand'),
    'records-brand-exchange': payloadChannels.includes('minecraft:brand'),
    'records-client-information': packetNames.includes('client_information'),
    'records-cookie-request-response': packetNames.includes('cookie_request') && packetNames.includes('cookie_response'),
    'records-configuration-disconnect': packetNames.includes('disconnect') || events.some(event => event.name === 'kicked')
  }

  const missing = plan.required.filter(requirement => !checks[requirement])
  return {
    ok: missing.length === 0,
    checks,
    missing
  }
}

export function summarizeConfigurationReplayEvidence(evidence, plan = createConfigurationReplayPlan()) {
  const official = evidence.official ?? {}
  const rustcraft = evidence.rustcraft ?? {}
  const officialMilestones = official.milestones ?? []
  const rustcraftMilestones = rustcraft.milestones ?? []

  const checks = {
    'records-official-configuration-transcript': Array.isArray(official.configPackets) && official.configPackets.length > 0,
    'records-rustcraft-configuration-transcript': Array.isArray(rustcraft.configPackets) && rustcraft.configPackets.length > 0,
    'matches-login-milestones': milestonesContainBoth(officialMilestones, rustcraftMilestones, ['tcp-connect', 'login-success']),
    'matches-configuration-milestones': milestonesContainBoth(officialMilestones, rustcraftMilestones, ['configuration-start', 'known-packs', 'finish-configuration']),
    'matches-play-entry-milestones': milestonesContainBoth(officialMilestones, rustcraftMilestones, ['play-login']),
    'no-hidden-sleeps': !(evidence.hiddenSleeps?.length > 0),
    'no-retry-only-success': evidence.retryOnlySuccess !== true
  }

  const missing = plan.required.filter(requirement => !checks[requirement])
  return {
    ok: missing.length === 0,
    checks,
    missing
  }
}

function milestonesContainBoth(official, rustcraft, milestones) {
  return milestones.every(milestone => official.includes(milestone) && rustcraft.includes(milestone))
}

function hasManifestKnownPack(packet, manifest) {
  return manifest.knownPacks.every(expected => {
    return packet.packs?.some(pack => {
      return pack.namespace === expected.namespace && pack.id === expected.id && pack.version === expected.version
    })
  })
}

function sameArray(actual, expected) {
  return actual.length === expected.length && actual.every((value, index) => value === expected[index])
}
