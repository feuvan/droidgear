import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type ConnectionDiagnostics,
  type ModelTestResult,
  type TestMode,
} from '@/lib/bindings'

interface ConnectivityState {
  testResults: ModelTestResult[]
  isLoading: boolean
  error: string | null
  lastTestedAt: Date | null
  testMode: TestMode
  customPrompt: string

  setTestMode: (mode: TestMode) => void
  setCustomPrompt: (prompt: string) => void
  testAllModels: () => Promise<void>
  testAllModelsWithMode: (
    mode: TestMode,
    prompt?: string | null
  ) => Promise<void>
  testSingleModel: (modelId: string) => Promise<void>
  testSingleModelWithMode: (
    modelId: string,
    mode: TestMode,
    prompt?: string | null
  ) => Promise<void>
  testProviderDirect: (
    provider: string,
    baseUrl: string,
    apiKey: string,
    modelId: string
  ) => Promise<ConnectionDiagnostics>
  clearResults: () => void
  setError: (error: string | null) => void
}

export const useConnectivityStore = create<ConnectivityState>()(
  devtools(
    (set, get) => ({
      testResults: [],
      isLoading: false,
      error: null,
      lastTestedAt: null,
      testMode: 'ping' as TestMode,
      customPrompt: 'Hi',

      setTestMode: mode => {
        set({ testMode: mode }, undefined, 'setTestMode')
      },

      setCustomPrompt: prompt => {
        set({ customPrompt: prompt }, undefined, 'setCustomPrompt')
      },

      testAllModels: async () => {
        const { testMode, customPrompt } = get()
        const { testAllModelsWithMode } = get()
        await testAllModelsWithMode(testMode, customPrompt)
      },

      testAllModelsWithMode: async (mode, prompt) => {
        set({ isLoading: true, error: null }, undefined, 'testAllModels/start')
        try {
          const result =
            mode === 'ping'
              ? await commands.testAllModelConnectionsCommand()
              : await commands.testAllModelConnectionsWithMode(
                  mode,
                  prompt ?? null
                )

          if (result.status === 'ok') {
            set(
              {
                testResults: result.data,
                isLoading: false,
                lastTestedAt: new Date(),
              },
              undefined,
              'testAllModels/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'testAllModels/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'testAllModels/exception'
          )
        }
      },

      testSingleModel: async (modelId: string) => {
        const { testMode, customPrompt } = get()
        const { testSingleModelWithMode } = get()
        await testSingleModelWithMode(modelId, testMode, customPrompt)
      },

      testSingleModelWithMode: async (modelId, mode, prompt) => {
        set(
          { isLoading: true, error: null },
          undefined,
          'testSingleModel/start'
        )
        try {
          const result =
            mode === 'ping'
              ? await commands.testModelConnection(modelId)
              : await commands.testModelConnectionWithMode(
                  modelId,
                  mode,
                  prompt ?? null
                )

          if (result.status === 'ok') {
            const currentResults = get().testResults
            const existingIndex = currentResults.findIndex(
              r => r.modelId === modelId
            )

            let updatedResults: ModelTestResult[]
            if (existingIndex >= 0) {
              updatedResults = [...currentResults]
              updatedResults[existingIndex] = result.data
            } else {
              updatedResults = [...currentResults, result.data]
            }

            set(
              {
                testResults: updatedResults,
                isLoading: false,
                lastTestedAt: new Date(),
              },
              undefined,
              'testSingleModel/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'testSingleModel/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'testSingleModel/exception'
          )
        }
      },

      testProviderDirect: async (provider, baseUrl, apiKey, modelId) => {
        set(
          { isLoading: true, error: null },
          undefined,
          'testProviderDirect/start'
        )
        try {
          const result = await commands.testProviderConnection(
            provider,
            baseUrl,
            apiKey,
            modelId
          )
          if (result.status === 'ok') {
            set({ isLoading: false }, undefined, 'testProviderDirect/success')
            return result.data
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'testProviderDirect/error'
            )
            throw new Error(result.error)
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'testProviderDirect/exception'
          )
          throw e
        }
      },

      clearResults: () => {
        set({ testResults: [], lastTestedAt: null }, undefined, 'clearResults')
      },

      setError: error => {
        set({ error }, undefined, 'setError')
      },
    }),
    { name: 'connectivity-store' }
  )
)
