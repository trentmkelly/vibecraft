import { assertMineflayerFixture } from './fixture_linter.mjs'
import { DEFAULT_FIXTURE_SEED, createScenarioFixture, profileSet } from './fixtures.mjs'

export const OFFLINE_BASELINE_PORT = 25_565
export const OFFLINE_BASELINE_VERSION = '1.21.6'

export async function createOfflineModeBaselineScenario(options = {}) {
  const botCount = options.botCount ?? 1
  const profiles = (options.profiles ?? profileSet(options.profilePrefix ?? 'VibeCraftBot', botCount))
    .map(profile => ({
      username: profile.username,
      expectedUuid: profile.uuid,
      fixtureRole: profile.fixtureRole,
      index: profile.index
    }))
  const fixture = await createScenarioFixture({
    name: options.name ?? 'offline-mode-baseline',
    root: options.root,
    port: options.port ?? OFFLINE_BASELINE_PORT,
    seed: options.seed ?? DEFAULT_FIXTURE_SEED,
    levelName: options.levelName ?? 'world',
    profiles,
    properties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      ...(options.properties ?? {})
    },
    keepArtifacts: options.keepArtifacts
  })
  const baseline = {
    ...fixture,
    version: options.version ?? OFFLINE_BASELINE_VERSION,
    serverProperties: fixture.properties,
    profiles,
    timeoutMs: options.timeoutMs ?? 30_000,
    packetCapture: {
      enabled: true,
      states: ['login', 'configuration', 'play']
    },
    vanillaComparison: {
      mode: options.vanillaComparisonMode ?? 'required',
      output: options.vanillaComparisonOutput ?? 'artifacts/mineflayer/offline-baseline-parity.json'
    }
  }
  assertMineflayerFixture(baseline)
  return baseline
}

export function summarizeOfflineBaselineScenario(baseline) {
  return {
    name: baseline.name,
    version: baseline.version,
    port: baseline.port,
    seed: baseline.seed,
    levelName: baseline.levelName,
    profiles: baseline.profiles.map(profile => ({
      username: profile.username,
      expectedUuid: profile.expectedUuid
    })),
    vanillaComparison: baseline.vanillaComparison
  }
}
