export function oracleFreshnessKey(input) {
  return stableJson({
    mineflayerVersion: input.mineflayerVersion,
    prismarineProtocolVersion: input.prismarineProtocolVersion,
    protocolVersion: input.protocolVersion,
    serverJarSha256: input.serverJarSha256,
    scenarioFixtureHash: input.scenarioFixtureHash
  })
}

export function evaluateOracleFreshness(current, cached) {
  const currentKey = oracleFreshnessKey(current)
  return {
    fresh: cached?.key === currentKey,
    key: currentKey,
    previousKey: cached?.key ?? null,
    rerunRequired: cached?.key !== currentKey
  }
}

function stableJson(value) {
  return JSON.stringify(Object.fromEntries(Object.entries(value).sort(([left], [right]) => left.localeCompare(right))))
}
