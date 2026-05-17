import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { offlineUuid, writeOfflineServerFiles } from './runner.mjs'

export const DEFAULT_FIXTURE_SEED = 8675309

export function deterministicProfile(name, index = 0) {
  const username = index === 0 ? name : `${name}${index}`
  return {
    username,
    uuid: offlineUuid(username),
    fixtureRole: name,
    index
  }
}

export function profileSet(prefix, count) {
  return Array.from({ length: count }, (_, index) => deterministicProfile(prefix, index))
}

export async function createScenarioFixture(options = {}) {
  const root = options.root ?? await mkdtemp(path.join(tmpdir(), `${options.name ?? 'rustcraft-scenario'}-`))
  await mkdir(root, { recursive: true })
  const port = options.port ?? 25_565
  const seed = options.seed ?? DEFAULT_FIXTURE_SEED
  const levelName = options.levelName ?? 'world'
  const profiles = options.profiles ?? profileSet('RustCraftBot', options.botCount ?? 1)
  const properties = await writeOfflineServerFiles(root, {
    port,
    seed,
    levelName,
    motd: options.motd ?? `RustCraft fixture ${options.name ?? 'scenario'}`,
    properties: options.properties
  })
  const logs = createLogCapture(options.name ?? 'scenario')
  return {
    name: options.name ?? 'scenario',
    root,
    port,
    seed,
    levelName,
    profiles,
    properties,
    logs,
    paths: {
      eula: path.join(root, 'eula.txt'),
      serverProperties: path.join(root, 'server.properties'),
      world: path.join(root, levelName),
      log: path.join(root, 'fixture.log')
    },
    cleanup: async () => {
      if (options.keepArtifacts !== true) await rm(root, { recursive: true, force: true })
    }
  }
}

export function createLogCapture(name = 'scenario') {
  const entries = []
  return {
    name,
    entries,
    write(stream, text) {
      entries.push({ stream, text, at: Date.now() })
    },
    attach(server) {
      server.logs.push = new Proxy(server.logs.push, {
        apply(target, thisArg, args) {
          for (const entry of args) entries.push({ ...entry, at: Date.now() })
          return Reflect.apply(target, thisArg, args)
        }
      })
      return server
    },
    normalized() {
      return entries.map(({ stream, text }) => ({ stream, text }))
    }
  }
}

export async function readFixtureFiles(fixture) {
  return {
    eula: await readFile(fixture.paths.eula, 'utf8'),
    serverProperties: await readFile(fixture.paths.serverProperties, 'utf8')
  }
}

export async function writeFixtureManifest(fixture) {
  const manifest = {
    name: fixture.name,
    root: fixture.root,
    port: fixture.port,
    seed: fixture.seed,
    levelName: fixture.levelName,
    profiles: fixture.profiles,
    properties: fixture.properties
  }
  const file = path.join(fixture.root, 'fixture-manifest.json')
  await writeFile(file, `${JSON.stringify(manifest, null, 2)}\n`)
  return file
}
