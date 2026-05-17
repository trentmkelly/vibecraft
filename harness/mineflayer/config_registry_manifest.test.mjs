import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const workspaceRoot = path.resolve(repoRoot, '..')

const registryDataLoaderPath = path.join(
  workspaceRoot,
  'decompiled-server-26.1.2',
  'net',
  'minecraft',
  'resources',
  'RegistryDataLoader.java'
)
const rawProbePath = path.join(here, 'raw_26_1_2_join_probe.mjs')

const documentedOmissions = new Map([
  [
    'minecraft:biome',
    'Minimal void-world join milestone has not implemented vanilla biome codec payloads or chunk biome palette parity yet.'
  ],
  [
    'minecraft:chat_type',
    'Chat/system-message registry sync is deferred until chat play packets and signed-message presentation are brought up together.'
  ],
  [
    'minecraft:enchantment',
    'Enchantment registry sync is deferred until item component initialization can be verified against vanilla item data.'
  ],
  [
    'minecraft:test_environment',
    'Game-test registries are deferred while validating the minimal client join path.'
  ],
  [
    'minecraft:test_instance',
    'Game-test registries are deferred while validating the minimal client join path.'
  ],
  [
    'minecraft:dialog',
    'Dialog registry sync is deferred until server-driven dialog packets are exercised in play-state tests.'
  ],
  [
    'minecraft:world_clock',
    'Experimental world clock registry sync is deferred until a vanilla transcript proves it is required for the current join milestone.'
  ],
  [
    'minecraft:timeline',
    'Experimental timeline registry sync is deferred until a vanilla transcript proves it is required for the current join milestone.'
  ]
])

function registryConstantToId (constant) {
  return `minecraft:${constant.toLowerCase()}`
}

function extractSynchronizedRegistries (source) {
  const block = source.match(/SYNCHRONIZED_REGISTRIES\s*=\s*List\.of\(([\s\S]*?)\n\s*\);/)
  assert.ok(block, 'could not find RegistryDataLoader.SYNCHRONIZED_REGISTRIES')

  return [...block[1].matchAll(/Registries\.([A-Z0-9_]+)/g)]
    .map(match => registryConstantToId(match[1]))
}

function extractProbeRegistries (source) {
  const block = source.match(/const expectedRegistries = \[([\s\S]*?)\]/)
  assert.ok(block, 'could not find raw probe expectedRegistries array')

  return [...block[1].matchAll(/'([^']+)'/g)].map(match => match[1])
}

test('raw 26.1.2 probe covers or documents every synchronized registry', async () => {
  const [registryDataLoader, rawProbe] = await Promise.all([
    readFile(registryDataLoaderPath, 'utf8'),
    readFile(rawProbePath, 'utf8')
  ])

  const vanillaRegistries = extractSynchronizedRegistries(registryDataLoader)
  const probedRegistries = new Set(extractProbeRegistries(rawProbe))
  const omittedRegistries = new Set(documentedOmissions.keys())

  const missing = vanillaRegistries.filter(registry => {
    return !probedRegistries.has(registry) && !omittedRegistries.has(registry)
  })
  assert.deepEqual(missing, [], 'synchronized registries must be probed or documented as intentionally omitted')

  const staleOmissions = [...omittedRegistries].filter(registry => {
    return !vanillaRegistries.includes(registry) || probedRegistries.has(registry)
  })
  assert.deepEqual(staleOmissions, [], 'documented omissions must exist in vanilla and must not already be probed')

  for (const [registry, reason] of documentedOmissions) {
    assert.ok(reason.length >= 40, `${registry} omission needs a useful reason`)
  }
})
