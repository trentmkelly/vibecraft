import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

const actionSets = [
  ['movement', 'chat', 'command_suggestion'],
  ['inventory_click', 'inventory_close', 'swing'],
  ['block_action', 'use_item_on', 'use_item', 'player_input']
]

test('raw 26.1.2 play-readiness race repeats immediate first actions without retry sleeps', { timeout: 90_000 }, async () => {
  for (const [index, actions] of actionSets.entries()) {
    const username = `Race${index}${actions.map(action => action[0]).join('').slice(0, 10)}`
    const result = await runJoinProbe(username, actions)

    assert.equal(result.ok, true)
    assert.equal(result.joinState.profile.name, username)
    assert.equal(result.joinState.lastReceivedChunk, 8, `${username} should receive the complete initial chunk batch`)
    assert.ok(result.keepAliveReplies > 0, `${username} should remain connected through the next keepalive`)
  }
})

async function runJoinProbe (username, actions) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        VIBECRAFT_USERNAME: username,
        VIBECRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: actions.join(','),
        VIBECRAFT_RAW_PROBE_KEEPALIVE_MS: '15000',
        ...(actions.includes('command_suggestion') ? { VIBECRAFT_EXPECT_COMMAND_SUGGESTION: 'list' } : {})
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}
