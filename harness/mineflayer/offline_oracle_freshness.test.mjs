import test from 'node:test'
import assert from 'node:assert/strict'
import {
  evaluateOracleFreshness,
  oracleFreshnessKey
} from './offline_oracle_freshness.mjs'

test('oracleFreshnessKey includes Mineflayer, prismarine, protocol, jar, and fixture inputs', () => {
  const key = oracleFreshnessKey(input())
  assert.match(key, /"mineflayerVersion":"4.28.0"/)
  assert.match(key, /"prismarineProtocolVersion":"1.21.6"/)
  assert.match(key, /"protocolVersion":775/)
  assert.match(key, /"serverJarSha256":"jar-sha"/)
  assert.match(key, /"scenarioFixtureHash":"fixture-sha"/)
})

test('evaluateOracleFreshness requires rerun when any freshness input changes', () => {
  const current = input()
  assert.deepEqual(evaluateOracleFreshness(current, { key: oracleFreshnessKey(current) }).rerunRequired, false)
  assert.equal(evaluateOracleFreshness({ ...current, serverJarSha256: 'new-jar' }, { key: oracleFreshnessKey(current) }).rerunRequired, true)
  assert.equal(evaluateOracleFreshness(current, null).rerunRequired, true)
})

function input() {
  return {
    mineflayerVersion: '4.28.0',
    prismarineProtocolVersion: '1.21.6',
    protocolVersion: 775,
    serverJarSha256: 'jar-sha',
    scenarioFixtureHash: 'fixture-sha'
  }
}
