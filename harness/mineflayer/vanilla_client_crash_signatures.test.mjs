import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const rawProbePath = path.join(here, 'raw_26_1_2_join_probe.mjs')

const configurationCrashSignatures = [
  {
    signature: 'Missing tag TagKey[minecraft:damage_type / minecraft:is_fire]',
    dependency: 'tag:minecraft:damage_type:minecraft:is_fire',
    initializer: 'Item.Properties.fireResistant'
  },
  {
    signature: 'Missing registry: ResourceKey[minecraft:root / minecraft:damage_type]',
    dependency: 'registry:minecraft:damage_type',
    initializer: 'RegistryDataCollector.resolveRegistryTags'
  },
  {
    signature: 'Registry must be non-empty: minecraft:cat_variant',
    dependency: 'registry:minecraft:cat_variant',
    initializer: 'RegistryDataLoader.RegistryValidator.nonEmpty'
  },
  {
    signature: 'Registry must be non-empty: minecraft:chicken_sound_variant',
    dependency: 'registry:minecraft:chicken_sound_variant',
    initializer: 'RegistryDataLoader.RegistryValidator.nonEmpty'
  },
  {
    signature: 'Failed to parse value for key minecraft:default from server',
    dependency: 'registry:minecraft:chicken_sound_variant:minecraft:default',
    initializer: 'ChickenSoundVariant.DIRECT_CODEC'
  },
  {
    signature: 'Missing element ResourceKey[minecraft:trim_material / minecraft:redstone]',
    dependency: 'registry:minecraft:trim_material:minecraft:redstone',
    initializer: 'Item.Properties.delayedHolderComponent'
  },
  {
    signature: 'Missing element ResourceKey[minecraft:chicken_variant / minecraft:cold]',
    dependency: 'registry:minecraft:chicken_variant:minecraft:cold',
    initializer: 'Item.Properties.delayedHolderComponent'
  },
  {
    signature: 'Missing element ResourceKey[minecraft:jukebox_song / minecraft:13]',
    dependency: 'registry:minecraft:jukebox_song:minecraft:13',
    initializer: 'Item.Properties.jukeboxPlayable'
  },
  {
    signature: 'Missing tag TagKey[minecraft:banner_pattern / minecraft:pattern_item/flower]',
    dependency: 'tag:minecraft:banner_pattern:minecraft:pattern_item/flower',
    initializer: 'Items.PROVIDES_BANNER_PATTERNS'
  },
  {
    signature: "Failed to decode packet 'clientbound/minecraft:player_position'",
    dependency: 'play-packet:clientbound/minecraft:player_position:length=62',
    initializer: 'ClientboundPlayerPositionPacket.STREAM_CODEC'
  }
]

function extractArrayBlock (source, name) {
  const match = source.match(new RegExp(`const ${name} = (?:new Map\\()?\\[([\\s\\S]*?)\\]\\)?`))
  assert.ok(match, `could not find ${name}`)
  return match[1]
}

function probeCoversDependency (source, dependency) {
  const parts = dependency.split(':')
  if (dependency.startsWith('registry:') && parts.length === 3) {
    return extractArrayBlock(source, 'expectedRegistries').includes(`'${parts.slice(1).join(':')}'`)
  }
  if (dependency.startsWith('registry:') && parts.length === 5) {
    const registry = parts.slice(1, 3).join(':')
    const element = parts.slice(3, 5).join(':')
    return source.includes(`['${registry}'`) && source.includes(`'${element}'`)
  }
  if (dependency.startsWith('tag:') && parts.length === 5) {
    const registry = parts.slice(1, 3).join(':')
    const tag = parts.slice(3, 5).join(':')
    return source.includes(`['${registry}'`) && source.includes(`'${tag}'`)
  }
  if (dependency === 'play-packet:clientbound/minecraft:player_position:length=62') {
    return source.includes('positionPacket.length !== 62')
  }
  return false
}

test('vanilla client configuration crash signatures are covered by raw probe requirements', async () => {
  const rawProbe = await readFile(rawProbePath, 'utf8')

  for (const crash of configurationCrashSignatures) {
    assert.ok(crash.signature.length > 20, 'crash signature should identify the vanilla failure')
    assert.ok(crash.initializer.length > 10, 'crash fixture should identify the decompiled initializer')
    assert.ok(
      probeCoversDependency(rawProbe, crash.dependency),
      `${crash.signature} is not covered by raw probe dependency ${crash.dependency}`
    )
  }
})
