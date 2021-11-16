import * as assert from 'node:assert'
import browserslist from 'browserslist'
// @ts-ignore
import { execute } from '../browserslist-rs.node'

describe('should match `browserslist` library', () => {
  it('current node', () => {
    assert.deepStrictEqual(
      execute('current node'),
      browserslist('current node')
    )
  })

  it('defaults', () => {
    assert.deepStrictEqual(execute('defaults'), browserslist('defaults'))
  })

  it('electron', () => {
    assert.deepStrictEqual(
      execute('electron >= 10.0'),
      browserslist('electron >= 10.0')
    )
  })
})
