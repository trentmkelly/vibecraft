export function createQuarantineReport(options = {}) {
  const session = options.session ?? {}
  const rawDisconnectPackets = rawDisconnects(session.packetTrace ?? [])
  const protocol = {
    requestedVersion: session.endpoint?.version ?? options.version ?? null,
    protocolVersion: session.bot?.version ?? session.protocolVersion ?? null
  }
  const dependencySnapshot = prismarineDependencySnapshot(options.packageLock)
  const classification = classifyFailure({
    error: session.error,
    milestones: options.milestones ?? [],
    rawDisconnectPackets
  })
  return {
    classification,
    protocol,
    dependencies: dependencySnapshot,
    rawDisconnectPackets,
    evidence: {
      error: session.error ? { message: session.error.message, stack: session.error.stack } : null,
      milestones: options.milestones ?? [],
      timeline: (session.timeline ?? []).map(event => ({ name: event.name, summary: event.summary }))
    }
  }
}

export function classifyFailure(options = {}) {
  const message = options.error?.message ?? ''
  const failedMilestones = (options.milestones ?? []).filter(milestone => !milestone.ok)
  const disconnectText = (options.rawDisconnectPackets ?? [])
    .map(packet => JSON.stringify(packet.data ?? packet))
    .join('\n')
  if (/unsupported protocol|no data available for version|unknown minecraft version/i.test(message)) {
    return reason('mineflayer_client_version', 'Mineflayer or minecraft-data does not support the requested version')
  }
  if (/outdated (client|server)|multiplayer\.disconnect\.outdated/i.test(disconnectText)) {
    return reason('mineflayer_client_version', 'Server reported an outdated client/server version mismatch')
  }
  if (failedMilestones.some(milestone => ['tcpReadiness', 'login', 'configurationOrdering', 'firstSpawn'].includes(milestone.name))) {
    return reason('server_bug', `Failed server milestone: ${failedMilestones[0].name}`)
  }
  if (failedMilestones.some(milestone => milestone.name === 'unexpectedDisconnect')) {
    return reason('server_bug', 'Unexpected disconnect during offline-mode login')
  }
  return reason('unknown', 'Insufficient evidence to classify failure')
}

export function prismarineDependencySnapshot(packageLock) {
  const packages = packageLock?.packages ?? {}
  return Object.fromEntries(
    Object.entries(packages)
      .filter(([name]) => /node_modules\/(mineflayer|prismarine-|minecraft-data)/.test(name))
      .map(([name, metadata]) => [name.replace(/^node_modules\//, ''), metadata.version])
      .sort(([left], [right]) => left.localeCompare(right))
  )
}

export function formatQuarantineReport(report) {
  return [
    `classification: ${report.classification.kind}`,
    `reason: ${report.classification.reason}`,
    `requestedVersion: ${report.protocol.requestedVersion ?? '<default>'}`,
    `protocolVersion: ${report.protocol.protocolVersion ?? '<unknown>'}`,
    `rawDisconnectPackets: ${report.rawDisconnectPackets.length}`,
    `dependencies: ${Object.entries(report.dependencies).map(([name, version]) => `${name}@${version}`).join(', ') || '<none>'}`
  ].join('\n')
}

function rawDisconnects(packetTrace) {
  return packetTrace.filter(packet =>
    ['disconnect', 'kick_disconnect'].includes(packet.name) ||
    packet.keys?.includes('reason') ||
    packet.keys?.includes('message')
  )
}

function reason(kind, text) {
  return { kind, reason: text }
}
