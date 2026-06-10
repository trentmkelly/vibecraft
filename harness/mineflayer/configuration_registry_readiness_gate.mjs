import { spawn } from 'node:child_process'

import { configurationCompletionManifest } from './configuration_completion_manifest.mjs'
import { runConfigurationRegistryClosureGate } from './configuration_registry_closure_report.mjs'

export const CONFIGURATION_REGISTRY_READINESS_CHECKS = [
  'configuration-registry-closure',
  'completion-manifest',
  'raw-26-play-entry-probe'
]

export function evaluateConfigurationRegistryReadinessGate (evidence) {
  const manifestGate = evaluateManifestReadiness(configurationCompletionManifest, evidence.rawProbe)
  const checks = [
    evidence.closureGate?.ok
      ? pass('configuration-registry-closure', summarizeClosureGate(evidence.closureGate))
      : fail('configuration-registry-closure', 'configuration registry closure failed', summarizeClosureGate(evidence.closureGate)),
    manifestIsComplete(configurationCompletionManifest)
      ? pass('completion-manifest', {
          registries: configurationCompletionManifest.registryOrder.length,
          tagRegistries: Object.keys(configurationCompletionManifest.requiredTags).length,
          knownPacks: configurationCompletionManifest.knownPacks
        })
      : fail('completion-manifest', 'configuration completion manifest is incomplete', configurationCompletionManifest),
    ...manifestGate
  ]

  return {
    ok: checks.every(check => check.ok),
    workflows: CONFIGURATION_REGISTRY_READINESS_CHECKS,
    checks
  }
}

export async function runConfigurationRegistryReadinessGate (options = {}) {
  const { gate: closureGate } = await runConfigurationRegistryClosureGate()
  const rawProbe = options.runRawProbe === false
    ? { ok: true, skipped: true, reason: 'raw probe skipped by caller' }
    : await runRaw26Probe(options)

  return evaluateConfigurationRegistryReadinessGate({ closureGate, rawProbe })
}

export function formatConfigurationRegistryReadinessGateReport (gate) {
  return gate.checks.map(check => {
    return `${check.ok ? 'PASS' : 'FAIL'} ${check.name}: ${check.message ?? JSON.stringify(check.details)}`
  }).join('\n')
}

export function manifestIsComplete (manifest) {
  return manifest.protocolVersion === 775 &&
    manifest.finishConfigurationPacketId === 3 &&
    manifest.registryOrder.length > 0 &&
    Object.keys(manifest.elementCounts).length === manifest.registryOrder.length &&
    manifest.knownPacks.some(pack => {
      return pack.namespace === 'minecraft' && pack.id === 'core' && pack.version === '26.1.2'
    }) &&
    manifest.playEntryPackets.includes('clientbound/minecraft:level_chunk_with_light')
}

export function evaluateManifestReadiness (manifest, rawProbe) {
  if (!rawProbe?.ok) {
    return [fail('raw-26-play-entry-probe', 'raw 26.1.2 probe did not prove configuration-to-play readiness', rawProbe)]
  }

  if (rawProbe.skipped) {
    return [pass('raw-26-play-entry-probe', { skipped: true, reason: rawProbe.reason })]
  }

  return [
    registryOrderCheck(manifest, rawProbe),
    registryElementCountCheck(manifest, rawProbe),
    requiredRegistryElementCheck(manifest, rawProbe),
    requiredTagCheck(manifest, rawProbe),
    knownPackCheck(manifest, rawProbe),
    finishConfigurationCheck(manifest, rawProbe),
    playEntryCheck(rawProbe),
    spawnChunkBatchFramingCheck(rawProbe)
  ]
}

function registryOrderCheck (manifest, rawProbe) {
  const actual = registryPackets(rawProbe).map(packet => packet.registry)
  return arrayEqual(actual, manifest.registryOrder)
    ? pass('raw-26-registry-order', { registries: actual.length })
    : fail('raw-26-registry-order', 'registry packet order drifted from the completion manifest', { expected: manifest.registryOrder, actual })
}

function registryElementCountCheck (manifest, rawProbe) {
  const actual = new Map(registryPackets(rawProbe).map(packet => [packet.registry, packet.elements]))
  const failures = Object.entries(manifest.elementCounts).filter(([registry, expected]) => actual.get(registry) !== expected)
  return failures.length === 0
    ? pass('raw-26-registry-element-counts', Object.fromEntries(actual))
    : fail('raw-26-registry-element-counts', 'registry element count mismatch or truncated registry payload', failures.map(([registry, expected]) => ({ registry, expected, actual: actual.get(registry) })))
}

function requiredRegistryElementCheck (manifest, rawProbe) {
  const packets = new Map(registryPackets(rawProbe).map(packet => [packet.registry, new Set(packet.elementIds)]))
  const missing = []
  for (const [registry, elements] of Object.entries(manifest.requiredElements)) {
    const actual = packets.get(registry) ?? new Set()
    for (const element of elements) {
      if (!actual.has(element)) missing.push({ registry, element })
    }
  }
  return missing.length === 0
    ? pass('raw-26-required-registry-elements', { registries: Object.keys(manifest.requiredElements).length })
    : fail('raw-26-required-registry-elements', 'client-referenced registry elements are missing', missing)
}

function requiredTagCheck (manifest, rawProbe) {
  const tagsPacket = rawProbe.config?.find(packet => packet.id === 13)
  const tagRegistries = new Map((tagsPacket?.registries ?? []).map(registry => [registry.registry, registry.tags]))
  const missing = []
  for (const [registry, tags] of Object.entries(manifest.requiredTags)) {
    const actualTags = new Map((tagRegistries.get(registry) ?? []).map(tag => [tag.tag, tag.entries]))
    for (const [tag, expectedEntries] of Object.entries(tags)) {
      const actualEntries = actualTags.get(tag)
      if (!actualEntries) {
        missing.push({ registry, tag, reason: 'missing tag' })
      } else if (!arrayEqual(actualEntries, expectedEntries)) {
        missing.push({ registry, tag, expectedEntries, actualEntries })
      }
    }
  }
  return missing.length === 0
    ? pass('raw-26-required-tags', { registries: Object.keys(manifest.requiredTags).length })
    : fail('raw-26-required-tags', 'required tag packets or tag entries are missing', missing)
}

function knownPackCheck (manifest, rawProbe) {
  const knownPacksPacket = rawProbe.config?.find(packet => packet.id === 14)
  const actual = knownPacksPacket?.packs ?? []
  return arrayEqual(actual.map(packKey), manifest.knownPacks.map(packKey))
    ? pass('raw-26-known-pack-order', actual)
    : fail('raw-26-known-pack-order', 'known pack packet drifted from the completion manifest', { expected: manifest.knownPacks, actual })
}

function finishConfigurationCheck (manifest, rawProbe) {
  const finish = rawProbe.config?.at(-1)
  return finish?.id === manifest.finishConfigurationPacketId
    ? pass('raw-26-finish-configuration', { id: finish.id })
    : fail('raw-26-finish-configuration', 'finish configuration packet was missing or not last', { expected: manifest.finishConfigurationPacketId, actual: finish?.id })
}

function playEntryCheck (rawProbe) {
  const actual = rawProbe.play?.map(packet => packet.id) ?? []
  const expectedPrefix = [49, 70, 10, 64, 105, 103, 104, 18, 96, 113, 72, 43, 97, 94, 95, 38, 38, 38, 38, 12]
  const loginPacket = rawProbe.play?.find(packet => packet.id === 49)
  const positionPacket = rawProbe.play?.find(packet => packet.id === 72)
  const firstChunkIndex = actual.indexOf(45)
  const chunkBatchFinishedIndex = actual.indexOf(11)
  const prefixOk = expectedPrefix.every((packetId, index) => actual[index] === packetId)
  const chunkWindowOk = firstChunkIndex >= expectedPrefix.length &&
    chunkBatchFinishedIndex > firstChunkIndex &&
    actual.slice(firstChunkIndex, chunkBatchFinishedIndex).every(packetId => packetId === 45)
  const loginLengthOk = loginPacket?.length >= 70
  const positionLengthOk = positionPacket?.length >= 62
  const ok = prefixOk && chunkWindowOk && loginLengthOk && positionLengthOk
  return ok
    ? pass('raw-26-play-entry-packets', {
        prefix: expectedPrefix,
        chunkCount: chunkBatchFinishedIndex - firstChunkIndex,
        loginLength: loginPacket.length,
        positionLength: positionPacket.length
      })
    : fail('raw-26-play-entry-packets', 'play-state packet decode boundary changed', {
        expectedPrefix,
        actual,
        firstChunkIndex,
        chunkBatchFinishedIndex,
        loginLength: loginPacket?.length,
        positionLength: positionPacket?.length
      })
}

function spawnChunkBatchFramingCheck (rawProbe) {
  const actual = rawProbe.play?.map(packet => packet.id) ?? []
  const firstChunkIndex = actual.indexOf(45)
  const chunkBatchFinishedIndex = actual.indexOf(11)
  const chunkPackets = firstChunkIndex === -1 || chunkBatchFinishedIndex === -1
    ? []
    : rawProbe.play.slice(firstChunkIndex, chunkBatchFinishedIndex)
  const ok = firstChunkIndex !== -1 &&
    chunkBatchFinishedIndex > firstChunkIndex &&
    chunkPackets.length >= 9 &&
    chunkPackets.every(packet => packet.id === 45 && packet.length > 1024)

  return ok
    ? pass('raw-26-spawn-chunk-batch-framing', {
        firstChunkIndex,
        chunkBatchFinishedIndex,
        chunkCount: chunkPackets.length,
        minChunkLength: Math.min(...chunkPackets.map(packet => packet.length))
      })
    : fail('raw-26-spawn-chunk-batch-framing', 'spawn chunk batch must be framed as chunk packets followed by chunk_batch_finished after registry/tag closure', {
        actual,
        firstChunkIndex,
        chunkBatchFinishedIndex,
        chunkLengths: chunkPackets.map(packet => packet.length)
      })
}

function registryPackets (rawProbe) {
  return rawProbe.config?.filter(packet => packet.id === 7) ?? []
}

function packKey (pack) {
  return `${pack.namespace}:${pack.id}:${pack.version}`
}

function arrayEqual (left, right) {
  return left.length === right.length && left.every((value, index) => value === right[index])
}

function summarizeClosureGate (gate) {
  if (!gate) return { present: false }
  return {
    checks: gate.checks.map(check => ({ name: check.name, ok: check.ok }))
  }
}

async function runRaw26Probe (options) {
  const env = {
    ...process.env,
    VIBECRAFT_HOST: options.host ?? process.env.VIBECRAFT_HOST ?? '127.0.0.1',
    VIBECRAFT_PORT: String(options.port ?? process.env.VIBECRAFT_PORT ?? 25565),
    VIBECRAFT_USERNAME: options.username ?? process.env.VIBECRAFT_USERNAME ?? 'VibeCraftGate'
  }
  const timeoutMs = options.timeoutMs ?? 30_000

  return await new Promise(resolve => {
    const child = spawn(process.execPath, ['raw_26_1_2_join_probe.mjs'], {
      cwd: new URL('.', import.meta.url),
      env,
      stdio: ['ignore', 'pipe', 'pipe']
    })
    let stdout = ''
    let stderr = ''
    let resolved = false
    const finish = result => {
      if (resolved) return
      resolved = true
      clearTimeout(timeout)
      resolve(result)
    }
    const timeout = setTimeout(() => {
      child.kill('SIGTERM')
      finish({ ok: false, timeout: true, stdout, stderr })
    }, timeoutMs)

    child.stdout.on('data', chunk => { stdout += chunk })
    child.stderr.on('data', chunk => { stderr += chunk })
    child.on('close', code => {
      if (code !== 0) {
        finish({ ok: false, code, stdout, stderr })
        return
      }

      try {
        const parsed = JSON.parse(stdout)
        finish({
          ok: parsed.ok === true,
          config: parsed.config ?? [],
          play: parsed.play ?? []
        })
      } catch (error) {
        finish({ ok: false, code, stdout, stderr, error: error.message })
      }
    })
  })
}

function pass (name, details) {
  return { ok: true, name, details }
}

function fail (name, message, details) {
  return { ok: false, name, message, details }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const gate = await runConfigurationRegistryReadinessGate({
    host: process.env.VIBECRAFT_HOST,
    port: Number(process.env.VIBECRAFT_PORT ?? 25565),
    timeoutMs: Number(process.env.VIBECRAFT_TIMEOUT_MS ?? 30_000)
  })
  const report = formatConfigurationRegistryReadinessGateReport(gate)
  if (gate.ok) {
    console.log(report)
  } else {
    console.error(report)
    process.exitCode = 1
  }
}
