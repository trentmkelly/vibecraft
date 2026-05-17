import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 first-action matrix remains connected without retry sleeps', { timeout: 240_000 }, async () => {
  const actions = [
    'movement',
    'chat',
    'command_suggestion',
    'inventory_click',
    'inventory_close',
    'block_action',
    'swing',
    'use_item_on',
    'use_item'
  ]

  for (const action of actions) {
    const result = await runJoinProbe(`Act${action.replaceAll('_', '').slice(0, 9)}`, action)
    assert.equal(result.ok, true)
    assert.equal(result.joinState.lastReceivedChunk, 8, `${action} should receive complete initial chunk batch`)
    assert.ok(result.keepAliveReplies > 0, `${action} should stay connected through the next keepalive`)
  }

  const combined = await runJoinProbe('ActCombined', actions.join(','))
  assert.equal(combined.ok, true)
  assert.equal(combined.joinState.lastReceivedChunk, 8)
  assert.ok(combined.keepAliveReplies > 0, 'combined first actions should stay connected through the next keepalive')
})

async function runJoinProbe (username, actions) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: username,
        RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: actions,
        RUSTCRAFT_RAW_PROBE_KEEPALIVE_MS: '15000'
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}
