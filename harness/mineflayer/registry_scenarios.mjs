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
          'same-bot-against-rustcraft-and-official',
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

export function compactRegistryDiff(officialPackets, rustCraftPackets) {
  const official = registrySummary(officialPackets)
  const rustCraft = registrySummary(rustCraftPackets)
  const diffs = []
  for (const key of new Set([...Object.keys(official), ...Object.keys(rustCraft)])) {
    if (JSON.stringify(official[key]) !== JSON.stringify(rustCraft[key])) {
      diffs.push({ registry: key, official: official[key] ?? null, rustCraft: rustCraft[key] ?? null })
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

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(createRegistryScenarioPlan(), null, 2))
}
