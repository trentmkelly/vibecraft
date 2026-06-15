import assert from 'node:assert/strict'
import test from 'node:test'

import {
  loadConfigurationRegistryCodecAudit,
  readCodecAuditSources
} from './configuration_registry_codec_audit.mjs'
import { configurationCompletionManifest } from './configuration_completion_manifest.mjs'
import {
  decompiledSourceRoot,
  missingJavaSourceReason
} from './optional_decompiled_source.mjs'

test('configuration registry codec audit covers every synchronized registry', {
  skip: !decompiledSourceRoot ? missingJavaSourceReason : false
}, async () => {
  const audit = await loadConfigurationRegistryCodecAudit()

  assert.equal(audit.length, 28, '26.1.2 synchronized registry count changed')
  assert.deepEqual(audit.filter(entry => !entry.sourceFile), [], 'each registry needs a decompiled codec source file')
  assert.deepEqual(audit.filter(entry => entry.shape !== 'network-compound' && entry.shape !== 'direct-compound'), [])
  assert.deepEqual(audit.filter(entry => entry.codecFieldSource !== 'decompiled-source'), [])

  const missingOmissionEvidence = audit.filter(entry => !entry.emitted && !entry.omission)
  assert.deepEqual(missingOmissionEvidence, [], 'omitted registries need documented milestone evidence')
})

test('configuration registry codec audit records field evidence for every synchronized registry', {
  skip: !decompiledSourceRoot ? missingJavaSourceReason : false
}, async () => {
  const audit = await loadConfigurationRegistryCodecAudit()

  const missingRequiredFieldEvidence = audit.filter(entry => entry.emitted && entry.requiredFields.length === 0)
  assert.deepEqual(missingRequiredFieldEvidence, [], 'each synchronized registry needs required codec fields from decomp')

  for (const entry of audit) {
    assert.ok(Array.isArray(entry.optionalFields), `${entry.registry} optional fields must be recorded`)
    assert.ok(Array.isArray(entry.holderFields), `${entry.registry} holder fields must be recorded`)
    assert.ok(Array.isArray(entry.tagFields), `${entry.registry} tag fields must be recorded`)
    assert.ok(
      entry.notes.length > 0 || (!entry.emitted && entry.omission),
      `${entry.registry} needs a note explaining manual/extracted codec evidence or omission evidence`
    )
  }
})

test('synced registry unit manifest asserts counts, client IDs, and codec NBT fields', {
  skip: !decompiledSourceRoot ? missingJavaSourceReason : false
}, async () => {
  const audit = await loadConfigurationRegistryCodecAudit()
  const synced = audit.filter(entry => entry.emitted)

  for (const entry of synced) {
    assert.equal(
      typeof configurationCompletionManifest.elementCounts[entry.registry],
      'number',
      `${entry.registry} must have an exact element count`
    )
    assert.ok(
      configurationCompletionManifest.elementCounts[entry.registry] > 0,
      `${entry.registry} must not be represented by an empty synced registry`
    )
    for (const element of configurationCompletionManifest.requiredElements[entry.registry] ?? []) {
      assert.equal(typeof element, 'string', `${entry.registry} client-referenced IDs must be explicit strings`)
      assert.ok(element.startsWith('minecraft:'), `${entry.registry} client-referenced ID ${element} must be namespaced`)
    }
    assert.ok(entry.requiredFields.length > 0, `${entry.registry} needs decompiled NBT field names`)
  }
})

test('configuration registry codec audit source files contain the audited codec declarations', {
  skip: !decompiledSourceRoot ? missingJavaSourceReason : false
}, async () => {
  const audit = await loadConfigurationRegistryCodecAudit()
  const sources = await readCodecAuditSources(audit)

  for (const entry of audit) {
    const source = sources.get(entry.sourceFile)
    assert.ok(source, `${entry.registry} source was not read`)
    const [, codecName] = entry.codec.split('.')
    assert.ok(source.includes(codecName), `${entry.registry} source must mention ${entry.codec}`)
  }
})

test('biome codec audit records the 26.1.2 network payload shape from decomp', {
  skip: !decompiledSourceRoot ? missingJavaSourceReason : false
}, async () => {
  const audit = await loadConfigurationRegistryCodecAudit()
  const biome = audit.find(entry => entry.registry === 'minecraft:worldgen/biome')
  const sources = await readCodecAuditSources([biome])
  const source = sources.get(biome.sourceFile)

  assert.deepEqual(biome.requiredFields, ['has_precipitation', 'temperature', 'downfall', 'effects', 'water_color'])
  assert.deepEqual(biome.optionalFields, [
    'temperature_modifier',
    'attributes',
    'foliage_color',
    'dry_foliage_color',
    'grass_color',
    'grass_color_modifier'
  ])
  assert.deepEqual(biome.directOnlyFields, ['generation_settings', 'spawners'])
  assert.ok(source.includes('Biome.ClimateSettings.CODEC.forGetter'))
  assert.ok(source.includes('EnvironmentAttributeMap.NETWORK_CODEC.optionalFieldOf("attributes"'))
  assert.ok(source.includes('BiomeSpecialEffects.CODEC.fieldOf("effects"'))
  assert.ok(source.includes('BiomeGenerationSettings.CODEC.forGetter'))
  assert.ok(source.includes('MobSpawnSettings.CODEC.forGetter'))
  assert.ok(source.includes('BiomeGenerationSettings.EMPTY'))
  assert.ok(source.includes('MobSpawnSettings.EMPTY'))
})

test('enchantment codec audit records holder-backed direct-codec fields from decomp', {
  skip: !decompiledSourceRoot ? missingJavaSourceReason : false
}, async () => {
  const audit = await loadConfigurationRegistryCodecAudit()
  const enchantment = audit.find(entry => entry.registry === 'minecraft:enchantment')
  const sources = await readCodecAuditSources([enchantment])
  const source = sources.get(enchantment.sourceFile)

  assert.deepEqual(enchantment.requiredFields, [
    'description',
    'supported_items',
    'weight',
    'max_level',
    'min_cost',
    'max_cost',
    'anvil_cost',
    'slots'
  ])
  assert.deepEqual(enchantment.optionalFields, ['primary_items', 'exclusive_set', 'effects'])
  assert.deepEqual(enchantment.holderFields, ['supported_items', 'primary_items', 'exclusive_set'])
  assert.ok(source.includes('ComponentSerialization.CODEC.fieldOf("description"'))
  assert.ok(source.includes('RegistryCodecs.homogeneousList(Registries.ITEM).fieldOf("supported_items"'))
  assert.ok(source.includes('RegistryCodecs.homogeneousList(Registries.ENCHANTMENT).optionalFieldOf("exclusive_set"'))
  assert.ok(source.includes('EnchantmentEffectComponents.CODEC.optionalFieldOf("effects"'))
})
