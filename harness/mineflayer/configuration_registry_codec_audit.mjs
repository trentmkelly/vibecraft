import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { documentedRegistryOmissions, loadConfigurationRegistryClosureReport } from './configuration_registry_closure_report.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const workspaceRoot = path.resolve(repoRoot, '..')
const decompRoot = path.join(workspaceRoot, 'decompiled-server-26.1.2', 'net', 'minecraft')

export const codecAuditSourceFiles = new Map([
  ['Biome.NETWORK_CODEC', path.join(decompRoot, 'world', 'level', 'biome', 'Biome.java')],
  ['ChatType.DIRECT_CODEC', path.join(decompRoot, 'network', 'chat', 'ChatType.java')],
  ['TrimPattern.DIRECT_CODEC', path.join(decompRoot, 'world', 'item', 'equipment', 'trim', 'TrimPattern.java')],
  ['TrimMaterial.DIRECT_CODEC', path.join(decompRoot, 'world', 'item', 'equipment', 'trim', 'TrimMaterial.java')],
  ['WolfVariant.NETWORK_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'wolf', 'WolfVariant.java')],
  ['WolfSoundVariant.NETWORK_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'wolf', 'WolfSoundVariant.java')],
  ['PigVariant.NETWORK_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'pig', 'PigVariant.java')],
  ['PigSoundVariant.NETWORK_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'pig', 'PigSoundVariant.java')],
  ['FrogVariant.NETWORK_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'frog', 'FrogVariant.java')],
  ['CatVariant.NETWORK_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'feline', 'CatVariant.java')],
  ['CatSoundVariant.NETWORK_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'feline', 'CatSoundVariant.java')],
  ['CowSoundVariant.DIRECT_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'cow', 'CowSoundVariant.java')],
  ['CowVariant.NETWORK_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'cow', 'CowVariant.java')],
  ['ChickenSoundVariant.DIRECT_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'chicken', 'ChickenSoundVariant.java')],
  ['ChickenVariant.NETWORK_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'chicken', 'ChickenVariant.java')],
  ['ZombieNautilusVariant.NETWORK_CODEC', path.join(decompRoot, 'world', 'entity', 'animal', 'nautilus', 'ZombieNautilusVariant.java')],
  ['PaintingVariant.DIRECT_CODEC', path.join(decompRoot, 'world', 'entity', 'decoration', 'painting', 'PaintingVariant.java')],
  ['DimensionType.NETWORK_CODEC', path.join(decompRoot, 'world', 'level', 'dimension', 'DimensionType.java')],
  ['DamageType.DIRECT_CODEC', path.join(decompRoot, 'world', 'damagesource', 'DamageType.java')],
  ['BannerPattern.DIRECT_CODEC', path.join(decompRoot, 'world', 'level', 'block', 'entity', 'BannerPattern.java')],
  ['Enchantment.DIRECT_CODEC', path.join(decompRoot, 'world', 'item', 'enchantment', 'Enchantment.java')],
  ['JukeboxSong.DIRECT_CODEC', path.join(decompRoot, 'world', 'item', 'JukeboxSong.java')],
  ['Instrument.DIRECT_CODEC', path.join(decompRoot, 'world', 'item', 'Instrument.java')],
  ['TestEnvironmentDefinition.DIRECT_CODEC', path.join(decompRoot, 'gametest', 'framework', 'TestEnvironmentDefinition.java')],
  ['GameTestInstance.DIRECT_CODEC', path.join(decompRoot, 'gametest', 'framework', 'GameTestInstance.java')],
  ['Dialog.DIRECT_CODEC', path.join(decompRoot, 'server', 'dialog', 'Dialog.java')],
  ['WorldClock.DIRECT_CODEC', path.join(decompRoot, 'world', 'clock', 'WorldClock.java')],
  ['Timeline.NETWORK_CODEC', path.join(decompRoot, 'world', 'timeline', 'Timeline.java')]
])

export const registryCodecAuditOverrides = new Map([
  ['minecraft:worldgen/biome', {
    shape: 'network-compound',
    requiredFields: ['has_precipitation', 'temperature', 'downfall', 'effects', 'water_color'],
    optionalFields: ['temperature_modifier', 'attributes', 'foliage_color', 'dry_foliage_color', 'grass_color', 'grass_color_modifier'],
    directOnlyFields: ['generation_settings', 'spawners'],
    holderFields: [],
    tagFields: [],
    notes: [
      'Biome.NETWORK_CODEC omits generation settings and mob spawn settings; those are present only in Biome.DIRECT_CODEC.',
      'The network payload is a compound containing climate settings, optional positional attributes, and special effects.'
    ]
  }],
  ['minecraft:enchantment', {
    shape: 'direct-compound',
    requiredFields: ['description', 'supported_items', 'weight', 'max_level', 'min_cost', 'max_cost', 'anvil_cost', 'slots'],
    optionalFields: ['primary_items', 'exclusive_set', 'effects'],
    holderFields: ['supported_items', 'primary_items', 'exclusive_set'],
    tagFields: [],
    notes: [
      'Enchantment.DIRECT_CODEC is registry-backed and can reference item holder sets and enchantment holder sets.',
      'Default item component initialization currently does not require a concrete enchantment holder before first play-state entry.'
    ]
  }]
])

export function createConfigurationRegistryCodecAudit (closureReport) {
  return closureReport.map(entry => {
    const override = registryCodecAuditOverrides.get(entry.registry)
    return {
      registry: entry.registry,
      status: entry.status,
      emitted: entry.emitted,
      codec: entry.codec,
      sourceFile: codecAuditSourceFiles.get(entry.codec) ?? null,
      shape: override?.shape ?? (entry.codec.includes('NETWORK_CODEC') ? 'network-compound' : 'direct-compound'),
      requiredFields: override?.requiredFields ?? [],
      optionalFields: override?.optionalFields ?? [],
      directOnlyFields: override?.directOnlyFields ?? [],
      holderFields: override?.holderFields ?? [],
      tagFields: override?.tagFields ?? [],
      omission: entry.emitted ? null : documentedRegistryOmissions.get(entry.registry),
      notes: override?.notes ?? []
    }
  })
}

export async function loadConfigurationRegistryCodecAudit () {
  const audit = createConfigurationRegistryCodecAudit(await loadConfigurationRegistryClosureReport())
  const sources = await readCodecAuditSources(audit)
  return audit.map(entry => enrichCodecAuditEntry(entry, sources.get(entry.sourceFile) ?? ''))
}

export async function readCodecAuditSources (audit) {
  const uniqueSources = [...new Set(audit.map(entry => entry.sourceFile).filter(Boolean))]
  const pairs = await Promise.all(uniqueSources.map(async sourceFile => {
    return [sourceFile, await readFile(sourceFile, 'utf8')]
  }))
  return new Map(pairs)
}

export function enrichCodecAuditEntry (entry, source) {
  const extracted = extractCodecFields(source)
  const requiredFields = entry.requiredFields.length > 0
    ? entry.requiredFields
    : extracted.requiredFields
  const optionalFields = unique([
    ...entry.optionalFields,
    ...extracted.optionalFields.filter(field => !entry.requiredFields.includes(field))
  ])
  const holderFields = unique([...entry.holderFields, ...extracted.holderFields])
  const tagFields = unique([...entry.tagFields, ...extracted.tagFields])
  const notes = [...entry.notes]
  if (entry.requiredFields.length === 0 && requiredFields.length > 0) {
    notes.push('Required fields were extracted from decompiled fieldOf(...) declarations.')
  }
  if (entry.optionalFields.length === 0 && optionalFields.length > 0) {
    notes.push('Optional/defaulted fields were extracted from decompiled optionalFieldOf(...) declarations.')
  }

  return {
    ...entry,
    requiredFields,
    optionalFields,
    holderFields,
    tagFields,
    codecFieldSource: source ? 'decompiled-source' : 'missing-source',
    notes
  }
}

export function extractCodecFields (source) {
  const requiredFields = []
  const optionalFields = []
  const holderFields = []
  const tagFields = []

  for (const match of source.matchAll(/(?:optional)?fieldOf\("([^"]+)"/g)) {
    const prefix = source.slice(Math.max(0, match.index - 96), match.index)
    const field = match[1]
    if (prefix.includes('optional')) {
      optionalFields.push(field)
    } else {
      requiredFields.push(field)
    }
    if (prefix.includes('RegistryCodecs') || prefix.includes('HolderSet') || prefix.includes('Holder.CODEC')) {
      holderFields.push(field)
    }
    if (prefix.includes('TagKey') || prefix.includes('TagCodec')) {
      tagFields.push(field)
    }
  }

  return {
    requiredFields: unique(requiredFields),
    optionalFields: unique(optionalFields),
    holderFields: unique(holderFields),
    tagFields: unique(tagFields)
  }
}

function unique (values) {
  return [...new Set(values)]
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(await loadConfigurationRegistryCodecAudit(), null, 2))
}
