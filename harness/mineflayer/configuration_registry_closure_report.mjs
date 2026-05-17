import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { configurationCompletionManifest } from './configuration_completion_manifest.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const workspaceRoot = path.resolve(repoRoot, '..')

export const registryDataLoaderPath = path.join(
  workspaceRoot,
  'decompiled-server-26.1.2',
  'net',
  'minecraft',
  'resources',
  'RegistryDataLoader.java'
)

export const rawProbePath = path.join(here, 'raw_26_1_2_join_probe.mjs')

export const documentedRegistryOmissions = new Map([
  [
    'minecraft:biome',
    {
      milestone: 'void-world join',
      evidence: [
        'Biome.NETWORK_CODEC still needs an encoder for climate settings, optional positional attributes, and special effects.',
        'Initial chunk biome palette parity has not yet been compared against an official server.jar transcript.'
      ],
      next: 'Emit codec-valid baseline biome entries and assert them in the raw probe before broad client compatibility is marked complete.'
    }
  ],
  [
    'minecraft:enchantment',
    {
      milestone: 'void-world join',
      evidence: [
        'The item initializer audit found no default item component dependency on a concrete enchantment holder before play-state entry.',
        'The raw play-entry probe validates that current registry omissions still reach first play packets.'
      ],
      next: 'Choose omitted, minimal, or full vanilla enchantment policy and add matching raw-probe assertions before enchanted item smoke tests.'
    }
  ],
  [
    'minecraft:test_environment',
    {
      milestone: 'void-world join',
      evidence: ['Game-test packets and commands are outside the current minimal login/play-entry path.'],
      next: 'Sync or transcript-verify omission before game-test command parity work.'
    }
  ],
  [
    'minecraft:test_instance',
    {
      milestone: 'void-world join',
      evidence: ['Game-test packets and commands are outside the current minimal login/play-entry path.'],
      next: 'Sync or transcript-verify omission before game-test command parity work.'
    }
  ],
  [
    'minecraft:dialog',
    {
      milestone: 'void-world join',
      evidence: ['Server-driven dialog packets are not exercised before first play-state entry.'],
      next: 'Sync or transcript-verify omission before dialog packet tests.'
    }
  ],
  [
    'minecraft:world_clock',
    {
      milestone: 'void-world join',
      evidence: ['No current first-join packet or item initializer requires a world clock element.'],
      next: 'Verify against official transcript before time/timeline feature work.'
    }
  ],
  [
    'minecraft:timeline',
    {
      milestone: 'void-world join',
      evidence: ['No current first-join packet or item initializer requires a timeline element.'],
      next: 'Verify against official transcript before timeline feature work.'
    }
  ]
])

const codecSourceByRegistry = new Map([
  ['minecraft:biome', 'Biome.NETWORK_CODEC'],
  ['minecraft:chat_type', 'ChatType.DIRECT_CODEC'],
  ['minecraft:trim_pattern', 'TrimPattern.DIRECT_CODEC'],
  ['minecraft:trim_material', 'TrimMaterial.DIRECT_CODEC'],
  ['minecraft:wolf_variant', 'WolfVariant.NETWORK_CODEC'],
  ['minecraft:wolf_sound_variant', 'WolfSoundVariant.NETWORK_CODEC'],
  ['minecraft:pig_variant', 'PigVariant.NETWORK_CODEC'],
  ['minecraft:pig_sound_variant', 'PigSoundVariant.NETWORK_CODEC'],
  ['minecraft:frog_variant', 'FrogVariant.NETWORK_CODEC'],
  ['minecraft:cat_variant', 'CatVariant.NETWORK_CODEC'],
  ['minecraft:cat_sound_variant', 'CatSoundVariant.NETWORK_CODEC'],
  ['minecraft:cow_sound_variant', 'CowSoundVariant.DIRECT_CODEC'],
  ['minecraft:cow_variant', 'CowVariant.NETWORK_CODEC'],
  ['minecraft:chicken_sound_variant', 'ChickenSoundVariant.DIRECT_CODEC'],
  ['minecraft:chicken_variant', 'ChickenVariant.NETWORK_CODEC'],
  ['minecraft:zombie_nautilus_variant', 'ZombieNautilusVariant.NETWORK_CODEC'],
  ['minecraft:painting_variant', 'PaintingVariant.DIRECT_CODEC'],
  ['minecraft:dimension_type', 'DimensionType.NETWORK_CODEC'],
  ['minecraft:damage_type', 'DamageType.DIRECT_CODEC'],
  ['minecraft:banner_pattern', 'BannerPattern.DIRECT_CODEC'],
  ['minecraft:enchantment', 'Enchantment.DIRECT_CODEC'],
  ['minecraft:jukebox_song', 'JukeboxSong.DIRECT_CODEC'],
  ['minecraft:instrument', 'Instrument.DIRECT_CODEC'],
  ['minecraft:test_environment', 'TestEnvironmentDefinition.DIRECT_CODEC'],
  ['minecraft:test_instance', 'GameTestInstance.DIRECT_CODEC'],
  ['minecraft:dialog', 'Dialog.DIRECT_CODEC'],
  ['minecraft:world_clock', 'WorldClock.DIRECT_CODEC'],
  ['minecraft:timeline', 'Timeline.NETWORK_CODEC']
])

function registryConstantToId (constant) {
  return `minecraft:${constant.toLowerCase()}`
}

export function extractSynchronizedRegistries (source) {
  const block = source.match(/SYNCHRONIZED_REGISTRIES\s*=\s*List\.of\(([\s\S]*?)\n\s*\);/)
  if (!block) throw new Error('could not find RegistryDataLoader.SYNCHRONIZED_REGISTRIES')

  return [...block[1].matchAll(/Registries\.([A-Z0-9_]+)/g)]
    .map(match => registryConstantToId(match[1]))
}

export function extractProbeRegistries (source) {
  const block = source.match(/const expectedRegistries = \[([\s\S]*?)\]/)
  if (!block) throw new Error('could not find raw probe expectedRegistries array')

  return new Set([...block[1].matchAll(/'([^']+)'/g)].map(match => match[1]))
}

function tagCountFor (registry) {
  return Object.keys(configurationCompletionManifest.requiredTags[registry] ?? {}).length
}

function requiredElementCountFor (registry) {
  return (configurationCompletionManifest.requiredElements[registry] ?? []).length
}

export function createConfigurationRegistryClosureReport ({ registryDataLoader, rawProbe }) {
  const synchronizedRegistries = extractSynchronizedRegistries(registryDataLoader)
  const probeRegistries = extractProbeRegistries(rawProbe)

  return synchronizedRegistries.map((registry, index) => {
    const emitted = probeRegistries.has(registry)
    const omission = documentedRegistryOmissions.get(registry)
    const status = emitted ? 'synced' : 'documented_omission'

    return {
      registry,
      synchronizedIndex: index,
      status,
      emitted,
      codec: codecSourceByRegistry.get(registry) ?? 'unknown',
      expectedElements: configurationCompletionManifest.elementCounts[registry] ?? null,
      requiredElements: requiredElementCountFor(registry),
      requiredTags: tagCountFor(registry),
      validator: emitted ? 'raw_26_1_2_join_probe' : 'documented_omission',
      omission: emitted ? null : omission
    }
  })
}

export async function loadConfigurationRegistryClosureReport () {
  const [registryDataLoader, rawProbe] = await Promise.all([
    readFile(registryDataLoaderPath, 'utf8'),
    readFile(rawProbePath, 'utf8')
  ])

  return createConfigurationRegistryClosureReport({ registryDataLoader, rawProbe })
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const report = await loadConfigurationRegistryClosureReport()
  console.log(JSON.stringify(report, null, 2))
}
