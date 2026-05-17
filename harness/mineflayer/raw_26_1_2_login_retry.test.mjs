import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 offline login retry recovers immediately after a forced first-attempt disconnect', { timeout: 90_000 }, async () => {
  const phases = ['login_success', 'registry_sync', 'first_chunk']

  for (const phase of phases) {
    const username = `Retry${phase.replaceAll('_', '').slice(0, 10)}`
    const failed = await runJoinProbe(username, { RUSTCRAFT_RAW_PROBE_ABORT_AFTER: phase })
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
        RUSTCRAFT_USERNAME: username,
        ...env
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}
