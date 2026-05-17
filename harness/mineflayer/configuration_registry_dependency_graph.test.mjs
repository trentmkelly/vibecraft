import assert from 'node:assert/strict'
import test from 'node:test'

import {
  dependencyEvidence,
  loadConfigurationRegistryDependencyGraph,
  readDependencySources
} from './configuration_registry_dependency_graph.mjs'

test('configuration registry dependency graph covers all synchronized registries', async () => {
  const graph = await loadConfigurationRegistryDependencyGraph()

  assert.equal(graph.length, 28, '26.1.2 synchronized registry count changed')
  assert.deepEqual(graph.filter(entry => !entry.hasDependencyEvidence), [])
  assert.ok(graph.every(entry => entry.dependencies.includes('synchronized-registry-loader')))
})

test('dependency graph evidence is backed by decompiled source needles', async () => {
  const sources = await readDependencySources()

  for (const evidence of dependencyEvidence) {
    const source = sources.get(evidence.source)
    assert.ok(source, `${evidence.id} source was not loaded`)
    for (const needle of evidence.needles) {
      assert.ok(source.includes(needle), `${evidence.id} missing decompiled evidence: ${needle}`)
    }
  }
})

test('dependency graph identifies biome and dimension as play-entry dependencies', async () => {
  const graph = await loadConfigurationRegistryDependencyGraph()
  const biome = graph.find(entry => entry.registry === 'minecraft:biome')
  const dimensionType = graph.find(entry => entry.registry === 'minecraft:dimension_type')

  assert.ok(biome.dependencies.includes('chunk-biome-default'))
  assert.ok(biome.dependencies.includes('chunk-biome-palette-packet'))
  assert.ok(biome.dependencies.includes('level-chunk-section-biomes'))
  assert.ok(dimensionType.dependencies.includes('play-login-dimension-holder'))
  assert.ok(dimensionType.dependencies.includes('server-player-spawn-info'))
})

test('dependency graph identifies item initializer and enchantment follow-up dependencies', async () => {
  const graph = await loadConfigurationRegistryDependencyGraph()
  const trimMaterial = graph.find(entry => entry.registry === 'minecraft:trim_material')
  const jukeboxSong = graph.find(entry => entry.registry === 'minecraft:jukebox_song')
  const enchantment = graph.find(entry => entry.registry === 'minecraft:enchantment')

  assert.ok(trimMaterial.dependencies.includes('item-component-initializers'))
  assert.ok(jukeboxSong.dependencies.includes('item-component-initializers'))
  assert.ok(enchantment.dependencies.includes('enchantment-tooltip-order'))
})
