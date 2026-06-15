import test from 'node:test'
import assert from 'node:assert/strict'
import {
  compareRegistriesReports,
  createReportParityPlan,
  summarizeRegistriesReport
} from './report_parity_scenarios.mjs'

test('createReportParityPlan records official and VibeCraft report commands', () => {
  const plan = createReportParityPlan({ officialJar: '/tmp/official-server.jar' })

  assert.equal(plan.comparedAgainst, 'official-server.jar --report')
  assert.equal(plan.officialCommand, 'java -jar /tmp/official-server.jar --report')
  assert.equal(plan.reportPath, 'generated/reports/registries.json')
  assert.deepEqual(plan.requiredChecks, [
    'registry-id-set',
    'registry-entry-counts',
    'registry-entry-id-sets'
  ])
})

test('createReportParityPlan requires an explicit official server jar or command', () => {
  assert.throws(
    () => createReportParityPlan(),
    /VIBECRAFT_OFFICIAL_SERVER_JAR, options\.officialJar, or options\.officialCommand/
  )
})

test('summarizeRegistriesReport counts registry entries by registry id', () => {
  const summary = summarizeRegistriesReport({
    'minecraft:item': {
      entries: {
        'minecraft:stone': { protocol_id: 1 },
        'minecraft:dirt': { protocol_id: 2 }
      }
    },
    noise_settings: { ignored: true }
  })

  assert.deepEqual(summary, {
    'minecraft:item': {
      count: 2,
      ids: ['minecraft:dirt', 'minecraft:stone']
    }
  })
})

test('compareRegistriesReports passes matching registry ids and counts', () => {
  const report = {
    'minecraft:item': {
      entries: {
        'minecraft:stone': { protocol_id: 1 },
        'minecraft:dirt': { protocol_id: 2 }
      }
    },
    'minecraft:block': {
      entries: {
        'minecraft:stone': { protocol_id: 1 }
      }
    }
  }

  assert.deepEqual(compareRegistriesReports(report, report), {
    ok: true,
    registryCount: 2,
    differences: []
  })
})

test('compareRegistriesReports reports missing registries, count mismatches, and id mismatches', () => {
  const official = {
    'minecraft:item': {
      entries: {
        'minecraft:stone': { protocol_id: 1 },
        'minecraft:dirt': { protocol_id: 2 }
      }
    },
    'minecraft:block': {
      entries: {
        'minecraft:stone': { protocol_id: 1 }
      }
    }
  }
  const vibecraft = {
    'minecraft:item': {
      entries: {
        'minecraft:stone': { protocol_id: 1 },
        'minecraft:grass_block': { protocol_id: 3 }
      }
    },
    'minecraft:entity_type': {
      entries: {
        'minecraft:pig': { protocol_id: 1 }
      }
    }
  }

  const diff = compareRegistriesReports(official, vibecraft)
  assert.equal(diff.ok, false)
  assert.ok(diff.differences.some(entry => entry.kind === 'missing-registry'))
  assert.ok(diff.differences.some(entry => entry.kind === 'extra-registry'))
  assert.ok(diff.differences.some(entry => entry.kind === 'id-mismatch'))
})
