import { describe, it, expect } from 'vitest'
import { MODEL_OPTIONS } from './client'

describe('API Client', () => {
  describe('MODEL_OPTIONS', () => {
    it('should have 5 model options', () => {
      expect(MODEL_OPTIONS).toHaveLength(5)
    })

    it('should include claude-code option', () => {
      const claude = MODEL_OPTIONS.find(o => o.cli_agent === 'claude-code')
      expect(claude).toBeDefined()
      expect(claude?.model).toBe('claude-opus-4-5-20251101')
      expect(claude?.label).toBe('Claude Code / Opus 4.5')
    })

    it('should include codex option', () => {
      const codex = MODEL_OPTIONS.find(o => o.cli_agent === 'codex')
      expect(codex).toBeDefined()
      expect(codex?.model).toBe('gpt-5.2')
    })

    it('should include copilot option', () => {
      const copilot = MODEL_OPTIONS.find(o => o.cli_agent === 'copilot')
      expect(copilot).toBeDefined()
      expect(copilot?.model).toBe('claude-sonnet-4')
    })

    it('should include gemini option', () => {
      const gemini = MODEL_OPTIONS.find(o => o.cli_agent === 'gemini')
      expect(gemini).toBeDefined()
      expect(gemini?.model).toBe('gemini-3')
    })

    it('should include llr3 option', () => {
      const llr3 = MODEL_OPTIONS.find(o => o.cli_agent === 'llr3')
      expect(llr3).toBeDefined()
      expect(llr3?.model).toBe('llama3')
    })

    it('all options should have required fields', () => {
      MODEL_OPTIONS.forEach(option => {
        expect(option.cli_agent).toBeDefined()
        expect(option.model).toBeDefined()
        expect(option.label).toBeDefined()
        expect(typeof option.cli_agent).toBe('string')
        expect(typeof option.model).toBe('string')
        expect(typeof option.label).toBe('string')
      })
    })
  })
})
