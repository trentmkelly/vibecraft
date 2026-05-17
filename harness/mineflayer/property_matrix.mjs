import { offlineUuid } from './runner.mjs'

export function createOfflinePropertyMatrix(options = {}) {
  const username = options.username ?? 'RustCraftProperty'
  return {
    name: 'mineflayer-offline-property-matrix',
    mode: 'offline',
    auth: 'offline',
    profile: {
      username,
      expectedUuid: offlineUuid(username),
      fixture: 'generated-default-profile'
    },
    scenarios: [
      scenario('default-server-properties-login', {
        required: ['generated-profile', 'default-server.properties', 'first-join']
      }),
      scenario('offline-secure-default-world', {
        properties: {
          'online-mode': 'false',
          'enforce-secure-profile': 'false',
          'server-port': '<random>',
          'level-name': 'world'
        },
        required: ['default-port-selection', 'generated-world-directory']
      }),
      scenario('join-limits-and-status', {
        matrix: {
          'max-players': ['1', '20'],
          'enable-status': ['true', 'false'],
          'hide-online-players': ['true', 'false'],
          'network-compression-threshold': ['-1', '0', '256'],
          'player-idle-timeout': ['0', '1'],
          'white-list': ['false', 'true']
        },
        required: ['first-join-observation', 'vanilla-disconnect-or-accept']
      }),
      scenario('world-and-gameplay-initial-state', {
        matrix: {
          'level-name': ['world', 'custom_world'],
          'level-seed': ['8675309'],
          gamemode: ['survival', 'creative'],
          difficulty: ['easy', 'hard'],
          hardcore: ['false', 'true'],
          'force-gamemode': ['false', 'true'],
          'allow-flight': ['false', 'true'],
          'spawn-protection': ['0', '16']
        },
        required: ['vanilla-compatible-initial-state']
      }),
      scenario('single-property-login-bisect', {
        togglesOnePropertyPerRun: true,
        reportsFirstChangedBehavior: true
      }),
      scenario('generated-properties', {
        startsWith: ['deleted-server.properties', 'partial-server.properties'],
        required: ['generated-defaults', 'first-join-without-hand-edit']
      }),
      scenario('property-minimization', {
        removesOptionalKeysOneAtATime: true,
        required: ['vanilla-defaulting', 'warnings', 'first-login-behavior']
      }),
      scenario('negative-configuration', {
        cases: ['online-mode=true', 'secure-profile-enforcement', 'invalid-server-ip', 'occupied-port', 'malformed-properties'],
        required: ['vanilla-compatible-refusal-or-kick']
      }),
      scenario('empty-working-directory-login', {
        startsWith: [],
        required: ['eula.txt', 'server.properties', 'world-folders', 'first-successful-join', 'vanilla-order']
      }),
      scenario('property-roundtrip', {
        required: ['rewrite-server.properties', 'restart', 'same-offline-uuid', 'observed-config-changes']
      }),
      scenario('bind-address', {
        hosts: ['localhost', 'server-ip', 'randomized-port', 'rejected-address'],
        required: ['vanilla-compatible-socket-or-kick']
      }),
      scenario('compression-property-login', {
        thresholds: ['-1', '0', '32', '256'],
        required: ['first-join', 'packet-order-preserved']
      })
    ]
  }
}

export function summarizePropertyMatrixEvidence(evidence, matrix = createOfflinePropertyMatrix()) {
  const results = matrix.scenarios.map(entry => {
    const observed = evidence[entry.name] ?? {}
    const required = requiredEvidence(entry)
    const missing = required.filter(key => !observed[key])
    return {
      name: entry.name,
      ok: missing.length === 0,
      missing
    }
  })
  return {
    ok: results.every(result => result.ok),
    scenarios: results
  }
}

function scenario(name, details) {
  return { name, ...details }
}

function requiredEvidence(entry) {
  return [
    ...(entry.required ?? []),
    ...(entry.togglesOnePropertyPerRun ? ['toggles-one-property-per-run'] : []),
    ...(entry.reportsFirstChangedBehavior ? ['reports-first-changed-behavior'] : []),
    ...(entry.removesOptionalKeysOneAtATime ? ['removes-optional-keys-one-at-a-time'] : [])
  ]
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(createOfflinePropertyMatrix(), null, 2))
}
