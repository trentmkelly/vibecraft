export function createRegistryScenarioPlan() {
  return {
    name: 'mineflayer-offline-registry-scenarios',
    mode: 'offline',
    auth: 'offline',
    scenarios: [
      {
        name: 'registry-sync',
        required: [
          'captures-configuration-packets',
          'compares-registry-ids',
          'compares-tag-contents',
          'compares-known-packs',
          'compares-enabled-feature-order',
          'official-server-oracle'
        ]
      },
      {
        name: 'registry-login-diff',
        required: [
          'same-bot-against-vibecraft-and-official',
          'compact-registry-diff-on-play-state-failure',
          'configuration-diff-on-play-state-failure'
        ]
      },
      {
        name: 'registry-size-guard',
        required: [
          'large-registry-payload',
          'large-tag-payload',
          'configuration-completes',
          'no-mineflayer-parser-errors',
          'no-truncated-packets',
          'no-compression-regression'
        ]
      }
    ]
  }
}

export function summarizeRegistryEvidence(evidence, plan = createRegistryScenarioPlan()) {
  const scenarios = plan.scenarios.map(scenario => {
    const observed = evidence[scenario.name] ?? {}
    const missing = scenario.required.filter(key => !observed[key])
    return {
      name: scenario.name,
      ok: missing.length === 0,
      missing
    }
  })
  return {
    ok: scenarios.every(result => result.ok),
    scenarios
  }
}

export function buildRegistrySyncEvidence({ officialTranscript, vibeCraftTranscript }) {
  return {
    'captures-configuration-packets': hasConfigurationPackets(officialTranscript) && hasConfigurationPackets(vibeCraftTranscript),
    'compares-registry-ids': sameJson(
      vibeCraftTranscript.registries?.map(entry => entry.registry),
      officialTranscript.registries?.map(entry => entry.registry)
    ),
    'compares-tag-contents': sameJson(
      tagSummary(vibeCraftTranscript.tags),
      tagSummary(officialTranscript.tags)
    ),
    'compares-known-packs': sameJson(
      vibeCraftTranscript.knownPacks,
      officialTranscript.knownPacks
    ),
    'compares-enabled-feature-order': sameJson(
      enabledFeatures(vibeCraftTranscript),
      enabledFeatures(officialTranscript)
    ),
    'official-server-oracle': officialTranscript.source === 'official-server.jar' || Boolean(officialTranscript.officialOracle)
  }
}

export function buildRegistryLoginDiffEvidence({ officialTranscript, vibeCraftTranscript, vibeCraftPlayStateEntered }) {
  const diffs = compactRegistryDiff(
    transcriptPackets(officialTranscript),
    transcriptPackets(vibeCraftTranscript)
  )
  return {
    'same-bot-against-vibecraft-and-official': officialTranscript.username === vibeCraftTranscript.username,
    'compact-registry-diff-on-play-state-failure': vibeCraftPlayStateEntered || diffs.length > 0,
    'configuration-diff-on-play-state-failure': vibeCraftPlayStateEntered || Boolean(vibeCraftTranscript.disconnectReason || diffs.length > 0),
    diffs
  }
}

export function buildRegistrySizeGuardEvidence({ transcript, parserErrors = [], compressionThreshold, expectedCompressionThreshold }) {
  const payloadBytes = transcript.configurationBytes ?? transcript.rawConfigurationBytes ?? 0
  const registryBytes = transcript.registryBytes ?? payloadBytes
  const tagBytes = transcript.tagBytes ?? payloadBytes
  return {
    'large-registry-payload': registryBytes > 0,
    'large-tag-payload': tagBytes > 0,
    'configuration-completes': transcript.finishConfigurationPacketId === 3 || transcript.configurationComplete === true,
    'no-mineflayer-parser-errors': parserErrors.length === 0,
    'no-truncated-packets': !transcript.truncatedPackets,
    'no-compression-regression': compressionThreshold === expectedCompressionThreshold,
    payloadBytes
  }
}

export function compactRegistryDiff(officialPackets, vibeCraftPackets) {
  const official = registrySummary(officialPackets)
  const vibeCraft = registrySummary(vibeCraftPackets)
  const diffs = []
  for (const key of new Set([...Object.keys(official), ...Object.keys(vibeCraft)])) {
    if (JSON.stringify(official[key]) !== JSON.stringify(vibeCraft[key])) {
      diffs.push({ registry: key, official: official[key] ?? null, vibeCraft: vibeCraft[key] ?? null })
    }
  }
  return diffs
}

function registrySummary(packets) {
  const summary = {}
  for (const packet of packets) {
    if (packet.name !== 'registry_data') continue
    const id = packet.registryId ?? packet.registryCodec?.id ?? 'registryCodec'
    summary[id] = {
      keys: packet.keys ?? [],
      tags: packet.tags?.length ?? packet.tagCount ?? 0,
      knownPacks: packet.knownPacks?.length ?? packet.knownPackCount ?? 0,
      enabledFeatures: packet.enabledFeatures ?? packet.features ?? []
    }
  }
  return summary
}

function transcriptPackets(transcript) {
  return (transcript.registries ?? []).map(registry => ({
    name: 'registry_data',
    registryId: registry.registry,
    keys: registry.elementIds ?? [],
    tags: tagsForRegistry(transcript.tags, registry.registry),
    knownPacks: transcript.knownPacks ?? [],
    enabledFeatures: enabledFeatures(transcript)
  }))
}

function hasConfigurationPackets(transcript) {
  return Array.isArray(transcript.registries) && transcript.registries.length > 0 &&
    Array.isArray(transcript.knownPacks)
}

function tagSummary(tags = []) {
  return tags.map(registry => ({
    registry: registry.registry,
    tags: (registry.tags ?? []).map(tag => ({
      tag: tag.tag,
      entries: tag.entries
    }))
  }))
}

function tagsForRegistry(tags = [], registryId) {
  return tags.find(entry => entry.registry === registryId)?.tags ?? []
}

function enabledFeatures(transcript) {
  return transcript.enabledFeatures ?? transcript.features ?? ['minecraft:vanilla']
}

function sameJson(left, right) {
  return JSON.stringify(left) === JSON.stringify(right)
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(createRegistryScenarioPlan(), null, 2))
}
