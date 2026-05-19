import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 reconnect during configuration cleans stale profile/session state', { timeout: 90_000 }, async () => {
  for (const phase of ['registry_sync', 'known_packs']) {
    const username = `Cfg${phase.replaceAll('_', '').slice(0, 6)}${crypto.randomUUID().replaceAll('-', '').slice(0, 6)}`
    const dropped = await runJoinProbe(username, { RUSTCRAFT_RAW_PROBE_ABORT_AFTER: phase })
    assert.equal(dropped.ok, true)
    assert.equal(dropped.aborted, true)
    assert.equal(dropped.phase, phase)
    assert.ok(dropped.configPacketCount > 0, `${phase} should abort after configuration starts`)

    const retry = await runJoinProbe(username)
    assert.equal(retry.ok, true, `${phase} retry should reach play`)
    assert.equal(retry.joinState.profile.name, username)
    assert.ok(retry.configPacketCount > 0, `${phase} retry should receive configuration packets`)
    assert.ok(retry.playPacketCount > 0, `${phase} retry should receive play packets`)
    assert.ok(retry.joinState.initialChunkCount > 0, `${phase} retry should receive initial chunks`)
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
        RUSTCRAFT_RAW_PROBE_OUTPUT: 'summary',
        ...env
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}
