import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { access, readFile, rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'

import {
  createTempWorld,
  offlineUuid,
  startRustCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const execFileAsync = promisify(execFile)
const here = new URL('.', import.meta.url)
const repoRoot = path.resolve(here.pathname, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'rustcraft')
const host = '127.0.0.1'

test('raw 26.1.2 case-variant offline profiles keep distinct UUIDs and files', { timeout: 45_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-case-profile-')
  const names = ['CaseProfile', 'caseprofile']
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      properties: {
        'view-distance': '4',
        'simulation-distance': '4'
      }
    })
    server = startRustCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const joins = []
    for (const username of names) {
      const joined = await runJoinProbe(port, username)
      assert.equal(joined.ok, true)
      assert.equal(joined.joinState.profile.name, username)
      assert.equal(joined.joinState.profile.uuid, offlineUuid(username))
      joins.push(joined)
    }

    assert.notEqual(joins[0].joinState.profile.uuid, joins[1].joinState.profile.uuid)

    const usercache = JSON.parse(await readFile(path.join(root, 'usercache.json'), 'utf8'))
    for (const username of names) {
      const matches = usercache.filter(entry => entry.name === username)
      assert.equal(matches.length, 1)
      assert.equal(matches[0].uuid, offlineUuid(username))
      await access(path.join(root, 'world', 'playerdata', `${offlineUuid(username)}.dat`))
    }
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function runJoinProbe (port, username) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        RUSTCRAFT_HOST: host,
        RUSTCRAFT_PORT: String(port),
        RUSTCRAFT_USERNAME: username
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}

async function reservePort () {
  const server = createServer()
  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(0, host, resolve)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}
