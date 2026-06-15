export function summarizeRegistriesReport(report) {
  return Object.fromEntries(Object.entries(report ?? {})
    .filter(([, value]) => value && typeof value === 'object' && value.entries)
    .map(([registryId, value]) => [registryId, {
      count: Object.keys(value.entries ?? {}).length,
      ids: Object.keys(value.entries ?? {}).sort()
    }]))
}

export function compareRegistriesReports(officialReport, vibecraftReport) {
  const official = summarizeRegistriesReport(officialReport)
  const vibecraft = summarizeRegistriesReport(vibecraftReport)
  const registryIds = [...new Set([...Object.keys(official), ...Object.keys(vibecraft)])].sort()
  const differences = []
  for (const registryId of registryIds) {
    const left = official[registryId]
    const right = vibecraft[registryId]
    if (!left) {
      differences.push({ registryId, kind: 'extra-registry', vibecraftCount: right.count })
      continue
    }
    if (!right) {
      differences.push({ registryId, kind: 'missing-registry', officialCount: left.count })
      continue
    }
    if (left.count !== right.count) {
      differences.push({
        registryId,
        kind: 'count-mismatch',
        officialCount: left.count,
        vibecraftCount: right.count
      })
    }
    const missingIds = left.ids.filter(id => !right.ids.includes(id))
    const extraIds = right.ids.filter(id => !left.ids.includes(id))
    if (missingIds.length > 0 || extraIds.length > 0) {
      differences.push({ registryId, kind: 'id-mismatch', missingIds, extraIds })
    }
  }
  return {
    ok: differences.length === 0,
    registryCount: registryIds.length,
    differences
  }
}

export function createReportParityPlan(options = {}) {
  const officialJar = options.officialJar ?? process.env.VIBECRAFT_OFFICIAL_SERVER_JAR
  if (!options.officialCommand && !officialJar) {
    throw new Error('report parity requires VIBECRAFT_OFFICIAL_SERVER_JAR, options.officialJar, or options.officialCommand')
  }

  return {
    name: 'report-registry-parity',
    comparedAgainst: 'official-server.jar --report',
    vibecraftCommand: options.vibecraftCommand ?? 'vibecraft --report',
    officialCommand: options.officialCommand ?? `java -jar ${officialJar} --report`,
    reportPath: 'generated/reports/registries.json',
    requiredChecks: [
      'registry-id-set',
      'registry-entry-counts',
      'registry-entry-id-sets'
    ]
  }
}
