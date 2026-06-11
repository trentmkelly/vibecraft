import test from 'node:test'
import assert from 'node:assert/strict'
import { offlineUuid } from './runner.mjs'
import { evaluateFixtureUuidSanity } from './offline_fixture_sanity.mjs'

test('evaluateFixtureUuidSanity proves usernames map to expected UUIDs before start and after both logins', () => {
  const fixture = { profiles: [{ username: 'SanityBot', expectedUuid: offlineUuid('SanityBot') }] }
  assert.deepEqual(evaluateFixtureUuidSanity(fixture, {
    vanilla: { SanityBot: offlineUuid('SanityBot') },
    vibecraft: { SanityBot: offlineUuid('SanityBot') }
  }), {
    ok: true,
    results: [{
      username: 'SanityBot',
      expectedUuid: offlineUuid('SanityBot'),
      beforeStart: true,
      vanillaLogin: true,
      vibecraftLogin: true
    }]
  })
})

test('evaluateFixtureUuidSanity fails closed on stale vanilla or VibeCraft UUIDs', () => {
  const fixture = { profiles: [{ username: 'SanityBot', expectedUuid: offlineUuid('SanityBot') }] }
  const result = evaluateFixtureUuidSanity(fixture, {
    vanilla: { SanityBot: 'bad' },
    vibecraft: { SanityBot: offlineUuid('SanityBot') }
  })
  assert.equal(result.ok, false)
  assert.equal(result.results[0].vanillaLogin, false)
})
