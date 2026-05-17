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
    'play-entry-after-client-finish': [49, 105, 72, 12, 48, 11].every(id => playIds.includes(id))
  }

  const missing = plan.required.filter(requirement => !checks[requirement])
  return {
    ok: missing.length === 0,
    checks,
    missing
  }
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
