import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { loadConfigurationRegistryClosureReport } from './configuration_registry_closure_report.mjs'
import {
  decompiledSourceRoot,
  warnMissingJavaSource
} from './optional_decompiled_source.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const decompRoot = decompiledSourceRoot
  ? path.join(decompiledSourceRoot, 'net', 'minecraft')
  : null

function decompPath (...parts) {
  return decompRoot ? path.join(decompRoot, ...parts) : null
}

export const dependencySourceFiles = {
  registryDataLoader: decompPath('resources', 'RegistryDataLoader.java'),
  vanillaRegistries: decompPath('data', 'registries', 'VanillaRegistries.java'),
  items: decompPath('world', 'item', 'Items.java'),
  item: decompPath('world', 'item', 'Item.java'),
  itemEnchantments: decompPath('world', 'item', 'enchantment', 'ItemEnchantments.java'),
  palettedContainerFactory: decompPath('world', 'level', 'chunk', 'PalettedContainerFactory.java'),
  levelChunkSection: decompPath('world', 'level', 'chunk', 'LevelChunkSection.java'),
  chunksBiomesPacket: decompPath('network', 'protocol', 'game', 'ClientboundChunksBiomesPacket.java'),
  commonPlayerSpawnInfo: decompPath('network', 'protocol', 'game', 'CommonPlayerSpawnInfo.java'),
  serverPlayer: decompPath('server', 'level', 'ServerPlayer.java')
}

export const dependencyEvidence = [
  {
    id: 'synchronized-registry-loader',
    registries: 'all',
    source: 'registryDataLoader',
    needles: ['SYNCHRONIZED_REGISTRIES', 'Registries.BIOME', 'Registries.ENCHANTMENT', 'Registries.DIMENSION_TYPE'],
    reason: 'The configuration sync surface starts from RegistryDataLoader.SYNCHRONIZED_REGISTRIES.'
  },
  {
    id: 'vanilla-registry-bootstrap',
    registries: ['minecraft:worldgen/biome', 'minecraft:enchantment', 'minecraft:dimension_type'],
    source: 'vanillaRegistries',
    needles: [
      '.add(Registries.DIMENSION_TYPE, DimensionTypes::bootstrap)',
      '.add(Registries.BIOME, BiomeData::bootstrap)',
      '.add(Registries.ENCHANTMENT, Enchantments::bootstrap)'
    ],
    reason: 'The vanilla data bootstrap identifies the authoritative registry population path.'
  },
  {
    id: 'item-component-initializers',
    registries: ['minecraft:damage_type', 'minecraft:instrument', 'minecraft:jukebox_song', 'minecraft:trim_material', 'minecraft:chicken_variant'],
    source: 'items',
    needles: ['delayedHolderComponent', '.trimMaterial(', '.jukeboxPlayable(', 'context.getOrThrow'],
    reason: 'Default item component initializers caused the observed vanilla-client missing registry/tag crashes.'
  },
  {
    id: 'item-initializer-helper-tags',
    registries: ['minecraft:damage_type'],
    source: 'item',
    needles: ['fireResistant()', 'DamageTypeTags.IS_FIRE', 'DamageTypes.SPEAR'],
    reason: 'Item helper methods add indirect damage type registry and tag dependencies.'
  },
  {
    id: 'enchantment-tooltip-order',
    registries: ['minecraft:enchantment'],
    source: 'itemEnchantments',
    needles: ['Registries.ENCHANTMENT', 'EnchantmentTags.TOOLTIP_ORDER'],
    reason: 'Enchantment component rendering can consult enchantment tags once enchanted stacks are exercised.'
  },
  {
    id: 'chunk-biome-default',
    registries: ['minecraft:worldgen/biome'],
    source: 'palettedContainerFactory',
    needles: ['lookupOrThrow(Registries.BIOME)', 'Strategy.createForBiomes', 'Biomes.PLAINS'],
    reason: 'Chunk palette serialization needs a biome registry and vanilla plains default.'
  },
  {
    id: 'chunk-biome-palette-packet',
    registries: ['minecraft:worldgen/biome'],
    source: 'chunksBiomesPacket',
    needles: ['ClientboundChunksBiomesPacket', 'section.getBiomes().write(buffer)', 'getSerializedSize()'],
    reason: 'Clientbound chunk biome packets serialize section biome palettes using synced biome holder IDs.'
  },
  {
    id: 'level-chunk-section-biomes',
    registries: ['minecraft:worldgen/biome'],
    source: 'levelChunkSection',
    needles: ['PalettedContainerRO<Holder<Biome>> biomes', 'readBiomes', 'this.biomes.write(buffer)'],
    reason: 'Level chunk sections carry biome palettes that must resolve against the client registry.'
  },
  {
    id: 'play-login-dimension-holder',
    registries: ['minecraft:dimension_type'],
    source: 'commonPlayerSpawnInfo',
    needles: ['Holder<DimensionType> dimensionType', 'DimensionType.STREAM_CODEC', 'writeResourceKey(this.dimension)'],
    reason: 'Play login and respawn encode the dimension type as a registry holder.'
  },
  {
    id: 'server-player-spawn-info',
    registries: ['minecraft:dimension_type'],
    source: 'serverPlayer',
    needles: ['createCommonSpawnInfo', 'level.dimensionTypeRegistration()', 'level.dimension()'],
    reason: 'The server player join path constructs the common spawn info sent in play login.'
  }
]

export function createConfigurationRegistryDependencyGraph (closureReport) {
  return closureReport.map(entry => {
    const evidence = dependencyEvidence.filter(item => {
      return item.registries === 'all' || item.registries.includes(entry.registry)
    })

    return {
      registry: entry.registry,
      status: entry.status,
      emitted: entry.emitted,
      dependencies: evidence.map(item => item.id),
      hasDependencyEvidence: evidence.length > 0
    }
  })
}

export async function loadConfigurationRegistryDependencyGraph () {
  return createConfigurationRegistryDependencyGraph(await loadConfigurationRegistryClosureReport())
}

export async function readDependencySources () {
  if (!decompRoot) {
    warnMissingJavaSource('configuration registry dependency source audit')
    return new Map()
  }
  const pairs = await Promise.all(Object.entries(dependencySourceFiles).map(async ([key, sourceFile]) => {
    return [key, await readFile(sourceFile, 'utf8')]
  }))
  return new Map(pairs)
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(await loadConfigurationRegistryDependencyGraph(), null, 2))
}
