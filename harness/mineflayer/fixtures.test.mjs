import test from 'node:test'
import assert from 'node:assert/strict'
import { readFile, rm } from 'node:fs/promises'
import {
  createLogCapture,
  createScenarioFixture,
  deterministicProfile,
  profileSet,
  readFixtureFiles,
  writeFixtureManifest
} from './fixtures.mjs'

test('deterministicProfile produces stable offline-mode UUIDs', () => {
  assert.deepEqual(deterministicProfile('Steve'), {
    username: 'Steve',
    uuid: '5627dd98-e6be-3c21-b8a8-e92344183641',
    fixtureRole: 'Steve',
    index: 0
  })
  assert.equal(profileSet('Bot', 3).map(profile => profile.username).join(','), 'Bot,Bot1,Bot2')
})

test('createScenarioFixture writes eula, seeded properties, paths, and profiles', async () => {
  const fixture = await createScenarioFixture({
    name: 'login-fixture',
    port: 30123,
    seed: 42,
    levelName: 'fixture-world',
    botCount: 2,
    properties: { difficulty: 'hard' }
  })
  try {
    const files = await readFixtureFiles(fixture)
    assert.equal(files.eula, 'eula=true\n')
    assert.match(files.serverProperties, /^server-port=30123$/m)
    assert.match(files.serverProperties, /^level-seed=42$/m)
    assert.match(files.serverProperties, /^level-name=fixture-world$/m)
    assert.match(files.serverProperties, /^difficulty=hard$/m)
    assert.equal(fixture.paths.world.endsWith('fixture-world'), true)
    assert.equal(fixture.profiles.length, 2)
    assert.equal(fixture.profiles[0].uuid, deterministicProfile('VibeCraftBot').uuid)
  } finally {
    await fixture.cleanup()
  }
})

test('log capture records per-test server output without timestamps in normalized view', () => {
  const logs = createLogCapture('unit')
  logs.write('stdout', 'ready\n')
  logs.write('stderr', 'warn\n')
  assert.deepEqual(logs.normalized(), [
    { stream: 'stdout', text: 'ready\n' },
    { stream: 'stderr', text: 'warn\n' }
  ])
})

test('fixture manifest preserves deterministic scenario metadata', async () => {
  const fixture = await createScenarioFixture({ name: 'manifest-fixture', port: 30124 })
  try {
    const file = await writeFixtureManifest(fixture)
    const manifest = JSON.parse(await readFile(file, 'utf8'))
    assert.equal(manifest.name, 'manifest-fixture')
    assert.equal(manifest.port, 30124)
    assert.equal(manifest.seed, 8675309)
    assert.equal(manifest.profiles[0].username, 'VibeCraftBot')
  } finally {
    await rm(fixture.root, { recursive: true, force: true })
  }
})
