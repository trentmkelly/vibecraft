import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
import { configurationCompletionManifest } from './configuration_completion_manifest.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const rawProbePath = path.join(here, 'raw_26_1_2_join_probe.mjs')

test('configuration completion manifest is covered by the raw 26.1.2 probe', async () => {
  const rawProbe = await readFile(rawProbePath, 'utf8')

  assert.equal(configurationCompletionManifest.protocolVersion, 775)
  assert.equal(configurationCompletionManifest.finishConfigurationPacketId, 3)

  for (const registry of configurationCompletionManifest.registryOrder) {
    assert.ok(rawProbe.includes(`'${registry}'`), `raw probe does not require registry ${registry}`)
    assert.ok(
      Object.hasOwn(configurationCompletionManifest.elementCounts, registry),
      `manifest is missing element count for ${registry}`
    )
  }

  for (const [registry, count] of Object.entries(configurationCompletionManifest.elementCounts)) {
    assert.ok(configurationCompletionManifest.registryOrder.includes(registry), `count for unknown registry ${registry}`)
    if (count > 1) {
      assert.ok(rawProbe.includes(`['${registry}', ${count}]`), `raw probe does not require ${count} elements for ${registry}`)
    }
  }

  for (const [registry, elements] of Object.entries(configurationCompletionManifest.requiredElements)) {
    assert.ok(rawProbe.includes(`['${registry}'`), `raw probe has no required-elements block for ${registry}`)
    for (const element of elements) {
      assert.ok(rawProbe.includes(`'${element}'`), `raw probe does not require ${registry}/${element}`)
    }
  }

  for (const [registry, tags] of Object.entries(configurationCompletionManifest.requiredTags)) {
    assert.ok(rawProbe.includes(`['${registry}'`), `raw probe has no required-tags block for ${registry}`)
    for (const [tag, indices] of Object.entries(tags)) {
      assert.ok(rawProbe.includes(`'${tag}'`), `raw probe does not require tag ${registry}/${tag}`)
      assert.ok(indices.every(Number.isInteger), `tag ${registry}/${tag} has non-integer indices`)
    }
  }

  assert.ok(rawProbe.includes('positionPacket.length !== 62'), 'raw probe must validate player_position length')
})
