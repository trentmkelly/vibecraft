import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { readFile, rm, writeFile } from 'node:fs/promises'
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
const expiresOnPattern = /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} \+0000$/

const cases = [
  ['missing', 'CacheMissing', null],
  ['empty', 'CacheEmpty', ''],
  ['malformed', 'CacheMalformed', '[{"uuid":"broken"}\n'],
  ['stale', 'CacheStale', JSON.stringify([
    { uuid: '00000000-0000-0000-0000-000000000099', name: 'CacheStale', expiresOn: '2000-01-01 00:00:00 +0000' }
  ])],
  ['duplicate', 'CacheDuplicate', JSON.stringify([
    { uuid: offlineUuid('CacheDuplicate'), name: 'OldDuplicate', expiresOn: '2999-01-01 00:00:00 +0000' },
    { uuid: offlineUuid('CacheDuplicate'), name: 'CacheDuplicate', expiresOn: '2999-01-01 00:00:00 +0000' }
  ])]
]

for (const [label, username, initialUsercache] of cases) {
  test(`raw 26.1.2 repairs ${label} usercache before offline join`, { timeout: 45_000 }, async () => {
    await withServer(`rustcraft-usercache-${label}-`, initialUsercache, async ({ port, root }) => {
      const joined = await runJoinProbe(port, username)
      assert.equal(joined.ok, true)
      assert.equal(joined.joinState.profile.uuid, offlineUuid(username))

      const usercache = JSON.parse(await readFile(path.join(root, 'usercache.json'), 'utf8'))
      const matching = usercache.filter(entry => entry.name === username)
      assert.equal(matching.length, 1)
      assert.equal(matching[0].uuid, offlineUuid(username))
      assert.match(matching[0].expiresOn, expiresOnPattern)
      assert.equal(
        usercache.some(entry => entry.uuid === '00000000-0000-0000-0000-000000000099'),
        false
      )
    })
  })
}

async function withServer (prefix, initialUsercache, callback) {
  const port = await reservePort()
  const root = await createTempWorld(prefix)
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
    if (initialUsercache != null) {
      await writeFile(path.join(root, 'usercache.json'), `${initialUsercache}\n`)
    }

    server = startRustCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)
    return await callback({ port, root })
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
}

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
