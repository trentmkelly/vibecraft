import test from 'node:test'
import assert from 'node:assert/strict'
import {
  compareCommandTrees,
  createCommandTreeParityPlan,
  flattenCommandTree
} from './command_tree_parity.mjs'

test('createCommandTreeParityPlan covers command dump comparison surfaces', () => {
  assert.deepEqual(createCommandTreeParityPlan().surfaces, [
    'root-literals',
    'argument-types',
    'redirects',
    'permission-gates',
    'executable-flags'
  ])
})

test('flattenCommandTree emits stable paths with parser, redirect, permission, and executable metadata', () => {
  const nodes = flattenCommandTree({
    execute: {
      permission: 2,
      children: {
        as: {
          children: {
            targets: {
              kind: 'argument',
              parser: 'minecraft:entity',
              redirect: 'execute',
              executable: true
            }
          }
        }
      }
    }
  })

  assert.deepEqual(nodes.at(-1), {
    path: 'execute as targets',
    kind: 'argument',
    parser: 'minecraft:entity',
    redirect: 'execute',
    permission: 0,
    executable: true
  })
})

test('compareCommandTrees passes identical command dump surfaces', () => {
  const tree = {
    gamemode: {
      permission: 2,
      children: {
        mode: { kind: 'argument', parser: 'minecraft:game_mode', executable: true }
      }
    }
  }

  assert.deepEqual(compareCommandTrees(tree, tree), {
    ok: true,
    nodeCount: 2,
    differences: []
  })
})

test('compareCommandTrees reports root, argument, redirect, and permission mismatches', () => {
  const official = {
    execute: {
      permission: 2,
      children: {
        as: {
          children: {
            targets: { kind: 'argument', parser: 'minecraft:entity', redirect: 'execute' }
          }
        }
      }
    }
  }
  const rustcraft = {
    execute: {
      permission: 4,
      children: {
        as: {
          children: {
            targets: { kind: 'argument', parser: 'minecraft:players' }
          }
        }
      }
    },
    extra: {}
  }

  const diff = compareCommandTrees(official, rustcraft)
  assert.equal(diff.ok, false)
  assert.ok(diff.differences.some(entry => entry.kind === 'permission-mismatch'))
  assert.ok(diff.differences.some(entry => entry.kind === 'parser-mismatch'))
  assert.ok(diff.differences.some(entry => entry.kind === 'redirect-mismatch'))
  assert.ok(diff.differences.some(entry => entry.kind === 'extra-node'))
})
