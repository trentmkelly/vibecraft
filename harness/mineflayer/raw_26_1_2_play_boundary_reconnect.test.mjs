import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 play-boundary reconnects recover after forced disconnects', { timeout: 180_000 }, async () => {
  const phases = [
    ['join_game', {}],
    ['first_chunk', {}],
    ['chunk_batch_finished', {}],
    ['first_tick_actions', { VIBECRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: '1' }],
    ['first_keepalive', { VIBECRAFT_RAW_PROBE_KEEPALIVE_MS: '15000' }]
  ]

  for (const [phase, phaseEnv] of phases) {
    const username = `PRetry${phase.replaceAll('_', '').slice(0, 9)}`
    const failed = await runJoinProbe(username, {
      VIBECRAFT_RAW_PROBE_ABORT_AFTER: phase,
      ...phaseEnv
    })
    assert.equal(failed.ok, true)
    assert.equal(failed.aborted, true)
    assert.equal(failed.phase, phase)

    const retry = await runJoinProbe(username)
    assert.equal(retry.ok, true)
    assert.equal(retry.joinState.profile.name, username)
    assert.ok(retry.config.some(packet => packet.id === 3), `${phase} retry should finish configuration`)
    assert.ok(retry.play.some(packet => packet.id === 49), `${phase} retry should reach join game`)
    assert.equal(retry.joinState.lastReceivedChunk, 8, `${phase} retry should receive complete initial chunk batch`)
  }
})

async function runJoinProbe (username, env = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        VIBECRAFT_USERNAME: username,
        ...env
      },
      timeout: 45_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}
