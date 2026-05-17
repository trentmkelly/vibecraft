import assert from 'node:assert/strict'
import test from 'node:test'

import {
  loadConfigurationRegistryCodecAudit,
  readCodecAuditSources
} from './configuration_registry_codec_audit.mjs'

test('configuration registry codec audit covers every synchronized registry', async () => {
  const audit = await loadConfigurationRegistryCodecAudit()

  assert.equal(audit.length, 28, '26.1.2 synchronized registry count changed')
  assert.deepEqual(audit.filter(entry => !entry.sourceFile), [], 'each registry needs a decompiled codec source file')
  assert.deepEqual(audit.filter(entry => entry.shape !== 'network-compound' && entry.shape !== 'direct-compound'), [])

  const missingOmissionEvidence = audit.filter(entry => !entry.emitted && !entry.omission)
  assert.deepEqual(missingOmissionEvidence, [], 'omitted registries need documented milestone evidence')
})

test('configuration registry codec audit source files contain the audited codec declarations', async () => {
  const audit = await loadConfigurationRegistryCodecAudit()
  const sources = await readCodecAuditSources(audit)

  for (const entry of audit) {
    const source = sources.get(entry.sourceFile)
    assert.ok(source, `${entry.registry} source was not read`)
    const [, codecName] = entry.codec.split('.')
    assert.ok(source.includes(codecName), `${entry.registry} source must mention ${entry.codec}`)
  }
})

test('biome codec audit records the 26.1.2 network payload shape from decomp', async () => {
  const audit = await loadConfigurationRegistryCodecAudit()
  const biome = audit.find(entry => entry.registry === 'minecraft:biome')
  const sources = await readCodecAuditSources([biome])
  const source = sources.get(biome.sourceFile)

  assert.deepEqual(biome.requiredFields, ['has_precipitation', 'temperature', 'downfall', 'effects', 'water_color'])
  assert.ok(biome.optionalFields.includes('attributes'))
  assert.ok(source.includes('Biome.ClimateSettings.CODEC.forGetter'))
  assert.ok(source.includes('EnvironmentAttributeMap.NETWORK_CODEC.optionalFieldOf("attributes"'))
  assert.ok(source.includes('BiomeSpecialEffects.CODEC.fieldOf("effects"'))
  assert.ok(source.includes('BiomeGenerationSettings.EMPTY'))
  assert.ok(source.includes('MobSpawnSettings.EMPTY'))
})

test('enchantment codec audit records holder-backed direct-codec fields from decomp', async () => {
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
