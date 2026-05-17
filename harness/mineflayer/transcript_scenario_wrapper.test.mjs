import assert from 'node:assert/strict'
import test from 'node:test'

import {
  prismarineSupportsTargetProtocol,
  runTranscriptScenario,
  TARGET_PROTOCOL_VERSION
} from './transcript_scenario_wrapper.mjs'

test('prismarine support check remains false until protocol 775 is available', () => {
  assert.equal(prismarineSupportsTargetProtocol(), false)
  assert.equal(prismarineSupportsTargetProtocol({
    minecraftData: () => ({ version: { version: TARGET_PROTOCOL_VERSION } })
  }), true)
})

test('transcript scenario wrapper uses raw 26.1.2 probe while Mineflayer lacks target protocol', async () => {
  let rawCalled = false
  const result = await runTranscriptScenario({
    minecraftData: () => undefined,
    recordRawTranscript: async () => {
      rawCalled = true
      return { finishConfigurationPacketId: 3, playPacketIds: [49] }
    },
    runMineflayerScenario: async () => {
      throw new Error('Mineflayer should not run without target protocol support')
    }
  })

  assert.equal(rawCalled, true)
  assert.equal(result.runner, 'raw-26.1.2')
  assert.equal(result.protocolVersion, TARGET_PROTOCOL_VERSION)
  assert.deepEqual(result.transcript.playPacketIds, [49])
})

test('transcript scenario wrapper can switch to Mineflayer once protocol 775 is supported', async () => {
  let mineflayerCalled = false
  const result = await runTranscriptScenario({
    minecraftData: () => ({ version: { version: TARGET_PROTOCOL_VERSION } }),
    runMineflayerScenario: async (options) => {
      mineflayerCalled = true
      assert.equal(options.version, '26.1.2')
      return {
        rawProbe: {
          config: [{ id: 3 }],
          play: [{ id: 49 }]
        }
      }
    }
  })

  assert.equal(mineflayerCalled, true)
  assert.equal(result.runner, 'mineflayer')
  assert.equal(result.transcript.finishConfigurationPacketId, 3)
  assert.deepEqual(result.transcript.playPacketIds, [49])
})
