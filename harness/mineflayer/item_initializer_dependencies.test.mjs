import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const workspaceRoot = path.resolve(repoRoot, '..')

const itemsPath = path.join(
  workspaceRoot,
  'decompiled-server-26.1.2',
  'net',
  'minecraft',
  'world',
  'item',
  'Items.java'
)
const itemPath = path.join(
  workspaceRoot,
  'decompiled-server-26.1.2',
  'net',
  'minecraft',
  'world',
  'item',
  'Item.java'
)
const rawProbePath = path.join(here, 'raw_26_1_2_join_probe.mjs')

const resourceKeyOwners = new Map([
  ['ChickenVariants', 'minecraft:chicken_variant'],
  ['DamageTypes', 'minecraft:damage_type'],
  ['Instruments', 'minecraft:instrument'],
  ['JukeboxSongs', 'minecraft:jukebox_song'],
  ['TrimMaterials', 'minecraft:trim_material']
])

const tagOwners = new Map([
  ['BannerPatternTags', 'minecraft:banner_pattern'],
  ['DamageTypeTags', 'minecraft:damage_type']
])

const expectedItemInitializerDependencies = new Set([
  'registry:minecraft:chicken_variant:minecraft:cold',
  'registry:minecraft:chicken_variant:minecraft:temperate',
  'registry:minecraft:chicken_variant:minecraft:warm',
  'registry:minecraft:damage_type:minecraft:spear',
  'registry:minecraft:instrument:minecraft:ponder_goat_horn',
  'registry:minecraft:jukebox_song:minecraft:11',
  'registry:minecraft:jukebox_song:minecraft:13',
  'registry:minecraft:jukebox_song:minecraft:5',
  'registry:minecraft:jukebox_song:minecraft:blocks',
  'registry:minecraft:jukebox_song:minecraft:cat',
  'registry:minecraft:jukebox_song:minecraft:chirp',
  'registry:minecraft:jukebox_song:minecraft:creator',
  'registry:minecraft:jukebox_song:minecraft:creator_music_box',
  'registry:minecraft:jukebox_song:minecraft:far',
  'registry:minecraft:jukebox_song:minecraft:lava_chicken',
  'registry:minecraft:jukebox_song:minecraft:mall',
  'registry:minecraft:jukebox_song:minecraft:mellohi',
  'registry:minecraft:jukebox_song:minecraft:otherside',
  'registry:minecraft:jukebox_song:minecraft:pigstep',
  'registry:minecraft:jukebox_song:minecraft:precipice',
  'registry:minecraft:jukebox_song:minecraft:relic',
  'registry:minecraft:jukebox_song:minecraft:stal',
  'registry:minecraft:jukebox_song:minecraft:strad',
  'registry:minecraft:jukebox_song:minecraft:tears',
  'registry:minecraft:jukebox_song:minecraft:wait',
  'registry:minecraft:jukebox_song:minecraft:ward',
  'registry:minecraft:trim_material:minecraft:amethyst',
  'registry:minecraft:trim_material:minecraft:copper',
  'registry:minecraft:trim_material:minecraft:diamond',
  'registry:minecraft:trim_material:minecraft:emerald',
  'registry:minecraft:trim_material:minecraft:gold',
  'registry:minecraft:trim_material:minecraft:iron',
  'registry:minecraft:trim_material:minecraft:lapis',
  'registry:minecraft:trim_material:minecraft:netherite',
  'registry:minecraft:trim_material:minecraft:quartz',
  'registry:minecraft:trim_material:minecraft:redstone',
  'registry:minecraft:trim_material:minecraft:resin',
  'tag:minecraft:banner_pattern:minecraft:pattern_item/bordure_indented',
  'tag:minecraft:banner_pattern:minecraft:pattern_item/creeper',
  'tag:minecraft:banner_pattern:minecraft:pattern_item/field_masoned',
  'tag:minecraft:banner_pattern:minecraft:pattern_item/flower',
  'tag:minecraft:banner_pattern:minecraft:pattern_item/flow',
  'tag:minecraft:banner_pattern:minecraft:pattern_item/globe',
  'tag:minecraft:banner_pattern:minecraft:pattern_item/guster',
  'tag:minecraft:banner_pattern:minecraft:pattern_item/mojang',
  'tag:minecraft:banner_pattern:minecraft:pattern_item/piglin',
  'tag:minecraft:banner_pattern:minecraft:pattern_item/skull',
  'tag:minecraft:damage_type:minecraft:bypasses_shield',
  'tag:minecraft:damage_type:minecraft:is_explosion',
  'tag:minecraft:damage_type:minecraft:is_fire'
])

const constantNames = new Map([
  ['BORDURE_INDENTED', 'bordure_indented'],
  ['BYPASSES_SHIELD', 'bypasses_shield'],
  ['ELEVEN', '11'],
  ['FIELD_MASONED', 'field_masoned'],
  ['FIVE', '5'],
  ['IS_EXPLOSION', 'is_explosion'],
  ['IS_FIRE', 'is_fire'],
  ['THIRTEEN', '13']
])

function constantToId (constant) {
  return constantNames.get(constant) ?? constant.toLowerCase()
}

function extractProbeRegistries (source) {
  const block = source.match(/const expectedRegistries = \[([\s\S]*?)\]/)
  assert.ok(block, 'could not find raw probe expectedRegistries array')

  return new Set([...block[1].matchAll(/'([^']+)'/g)].map(match => match[1]))
}

function dependencyId (kind, registry, owner, constant) {
  const element = owner === 'DamageTypeTags' || owner === 'BannerPatternTags'
    ? constantToId(constant).replace(/^pattern_item_/, 'pattern_item/')
    : constantToId(constant)
  return `${kind}:${registry}:minecraft:${element}`
}

function extractItemDependencies (itemsSource, itemSource) {
  const dependencies = new Set()

  for (const match of itemsSource.matchAll(/delayedHolderComponent\([^,]+,\s*([A-Za-z]+)\.([A-Z0-9_]+)/g)) {
    const registry = resourceKeyOwners.get(match[1])
    assert.ok(registry, `unmapped delayedHolderComponent owner ${match[1]}`)
    dependencies.add(dependencyId('registry', registry, match[1], match[2]))
  }

  for (const match of itemsSource.matchAll(/\.trimMaterial\(([A-Za-z]+)\.([A-Z0-9_]+)\)/g)) {
    const registry = resourceKeyOwners.get(match[1])
    assert.ok(registry, `unmapped trimMaterial owner ${match[1]}`)
    dependencies.add(dependencyId('registry', registry, match[1], match[2]))
  }

  for (const match of itemsSource.matchAll(/\.jukeboxPlayable\(([A-Za-z]+)\.([A-Z0-9_]+)\)/g)) {
    const registry = resourceKeyOwners.get(match[1])
    assert.ok(registry, `unmapped jukeboxPlayable owner ${match[1]}`)
    dependencies.add(dependencyId('registry', registry, match[1], match[2]))
  }

  for (const match of itemsSource.matchAll(/context\.getOrThrow\(([A-Za-z]+)\.([A-Z0-9_]+)\)/g)) {
    const registry = tagOwners.get(match[1]) ?? resourceKeyOwners.get(match[1])
    assert.ok(registry, `unmapped context.getOrThrow owner ${match[1]}`)
    const kind = tagOwners.has(match[1]) ? 'tag' : 'registry'
    dependencies.add(dependencyId(kind, registry, match[1], match[2]))
  }

  if (itemsSource.includes('.fireResistant()')) {
    assert.match(itemSource, /fireResistant\(\)[\s\S]*DamageTypeTags\.IS_FIRE/)
    dependencies.add('tag:minecraft:damage_type:minecraft:is_fire')
  }

  if (itemsSource.includes('.spear(')) {
    assert.match(itemSource, /spear\([\s\S]*DamageTypes\.SPEAR/)
    dependencies.add('registry:minecraft:damage_type:minecraft:spear')
  }

  return dependencies
}

test('decompiled item initializers map dynamic dependencies to configuration coverage', async () => {
  const [itemsSource, itemSource, rawProbe] = await Promise.all([
    readFile(itemsPath, 'utf8'),
    readFile(itemPath, 'utf8'),
    readFile(rawProbePath, 'utf8')
  ])

  const dependencies = extractItemDependencies(itemsSource, itemSource)
  assert.deepEqual(dependencies, expectedItemInitializerDependencies)

  const probedRegistries = extractProbeRegistries(rawProbe)
  const missingRegistryPackets = [...dependencies]
    .filter(dependency => dependency.startsWith('registry:'))
    .map(dependency => dependency.split(':').slice(1, 3).join(':'))
    .filter(registry => !probedRegistries.has(registry))
  assert.deepEqual([...new Set(missingRegistryPackets)].sort(), [])

  const coveredTagRegistries = new Set(['minecraft:banner_pattern', 'minecraft:damage_type'])
  const missingTagPackets = [...dependencies]
    .filter(dependency => dependency.startsWith('tag:'))
    .map(dependency => dependency.split(':').slice(1, 3).join(':'))
    .filter(registry => !coveredTagRegistries.has(registry))
  assert.deepEqual([...new Set(missingTagPackets)].sort(), [])
})
