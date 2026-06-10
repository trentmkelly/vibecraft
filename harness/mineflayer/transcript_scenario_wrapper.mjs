import minecraftData from 'minecraft-data'

import { normalizeConfigurationTranscript, recordServerConfigurationTranscript } from './configuration_transcript_oracle.mjs'
import { runOfflineLoginScenario } from './runner.mjs'

export const TARGET_MINECRAFT_VERSION = '26.1.2'
export const TARGET_PROTOCOL_VERSION = 775

export function prismarineSupportsTargetProtocol(options = {}) {
  const loader = options.minecraftData ?? minecraftData
  const candidates = [
    TARGET_MINECRAFT_VERSION,
    String(TARGET_PROTOCOL_VERSION),
    options.versionAlias
  ].filter(Boolean)

  for (const candidate of candidates) {
    try {
      const data = loader(candidate)
      if (data?.version?.version === TARGET_PROTOCOL_VERSION) return true
    } catch {
      // Unsupported versions are the expected state until prismarine updates.
    }
  }

  return false
}

export async function runTranscriptScenario(options = {}) {
  const supportsMineflayer = prismarineSupportsTargetProtocol(options)
  if (!supportsMineflayer || options.forceRawProbe === true) {
    const transcript = await (options.recordRawTranscript ?? recordServerConfigurationTranscript)(options)
    return {
      runner: 'raw-26.1.2',
      protocolVersion: TARGET_PROTOCOL_VERSION,
      transcript
    }
  }

  const scenario = await (options.runMineflayerScenario ?? runOfflineLoginScenario)({
    ...options,
    version: TARGET_MINECRAFT_VERSION
  })
  if (scenario.error) throw scenario.error

  return {
    runner: 'mineflayer',
    protocolVersion: TARGET_PROTOCOL_VERSION,
    transcript: normalizeConfigurationTranscript(scenario.rawProbe ?? scenario.artifacts?.rawProbe ?? {})
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const result = await runTranscriptScenario({
    host: process.env.VIBECRAFT_HOST,
    port: Number(process.env.VIBECRAFT_PORT ?? 25565),
    forceRawProbe: process.env.VIBECRAFT_FORCE_RAW_TRANSCRIPT === '1'
  })
  console.log(JSON.stringify(result, null, 2))
}
