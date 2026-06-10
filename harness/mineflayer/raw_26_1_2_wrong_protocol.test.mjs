import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 wrong-protocol probe keeps status usable and rejects login', async () => {
  const status = await runProbe('raw_26_1_2_status_probe.mjs', {
    VIBECRAFT_PROTOCOL_VERSION: '1'
  })
  assert.equal(status.ok, true)
  assert.equal(status.status.version.protocol, 775)

  const login = await runProbe('raw_26_1_2_join_probe.mjs', {
    VIBECRAFT_PROTOCOL_VERSION: '1',
    VIBECRAFT_EXPECT_LOGIN_DISCONNECT: '1',
    VIBECRAFT_USERNAME: 'WrongProto'
  })
  assert.equal(login.ok, true)
  assert.equal(login.disconnected, true)
  assert.equal(login.login, 0)
})

async function runProbe (script, env) {
  const { stdout } = await execFileAsync(
    process.execPath,
    [script],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        ...env
      },
      timeout: 30000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}
