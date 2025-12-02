import { useState } from 'react'
import { useScramble } from 'use-scramble'

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000'

interface BatchResult {
  tx_hash: string
  success: boolean
  confirmations?: number
  consensus_reached?: boolean
  agreement_ratio?: number
  error?: string
}

interface BatchResponse {
  total: number
  successful: number
  failed: number
  results: BatchResult[]
  total_time_ms: number
}

export default function BatchVerifier() {
  const [txHashes, setTxHashes] = useState('')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<BatchResponse | null>(null)

  const { ref: titleRef } = useScramble({
    text: 'Batch Verification',
    speed: 0.6,
    tick: 1,
    step: 1,
    scramble: 4,
    seed: 0,
    playOnMount: true,
  })

  const handleBatchVerify = async () => {
    const hashes = txHashes
      .split('\n')
      .map(h => h.trim())
      .filter(h => h.length > 0)

    if (hashes.length === 0) return

    setLoading(true)
    setResult(null)

    try {
      const response = await fetch(`${API_URL}/verify-batch`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ tx_hashes: hashes }),
      })
      const data = await response.json()
      if (data.success) {
        setResult(data.data)
      }
    } catch (err) {
      console.error(err)
    } finally {
      setLoading(false)
    }
  }

  const useExample = () => {
    const example = `2916ab4fad38bafc0632716ed431d24e91213e23233ce99583962ed82ffed10e
2916ab4fad38bafc0632716ed431d24e91213e23233ce99583962ed82ffed10e`
    setTxHashes(example)
  }

  return (
    <div className="glass-card p-8 fade-in-up">
      <h2 className="text-3xl font-bold mb-6">
        <span className="text-zcash-green scramble-text" ref={titleRef} />
      </h2>

      <div className="mb-6 space-y-3">
        <textarea
          value={txHashes}
          onChange={(e) => setTxHashes(e.target.value)}
          placeholder="Enter transaction hashes (one per line, max 100)..."
          className="w-full glass-input font-mono text-sm min-h-[120px] resize-y"
        />
        <div className="flex gap-3">
          <button onClick={useExample} className="glass-button flex-1">
            Load Example
          </button>
          <button
            onClick={handleBatchVerify}
            disabled={loading}
            className="glass-button-primary flex-1"
          >
            {loading ? 'Verifying...' : 'Verify Batch'}
          </button>
        </div>
      </div>

      {result && (
        <div className="space-y-4 fade-in-scale">
          {/* Stats */}
          <div className="grid grid-cols-3 gap-3">
            <div className="glass-card p-4 text-center">
              <div className="text-3xl font-bold text-zcash-green mb-1">
                {result.total}
              </div>
              <div className="text-xs text-gray-500">Total</div>
            </div>
            <div className="glass-card p-4 text-center">
              <div className="text-3xl font-bold text-blue-400 mb-1">
                {result.successful}
              </div>
              <div className="text-xs text-gray-500">Success</div>
            </div>
            <div className="glass-card p-4 text-center">
              <div className="text-3xl font-bold text-red-400 mb-1">
                {result.failed}
              </div>
              <div className="text-xs text-gray-500">Failed</div>
            </div>
          </div>

          {/* Time */}
          <div className="glass-card p-4 font-mono text-center">
            <span className="text-gray-500">Total Time: </span>
            <span className="text-purple-400 font-bold">
              {result.total_time_ms}ms
            </span>
            <span className="text-gray-500 ml-4">Avg: </span>
            <span className="text-blue-400">
              {(result.total_time_ms / result.total).toFixed(0)}ms/tx
            </span>
          </div>

          {/* Results */}
          <div className="max-h-[400px] overflow-y-auto space-y-2 pr-2">
            {result.results.map((tx, idx) => (
              <div
                key={idx}
                className={`glass-card p-3 fade-in-delay-${Math.min(idx + 1, 5)}`}
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="flex-1 min-w-0">
                    <div className="font-mono text-xs text-gray-400 truncate">
                      {tx.tx_hash}
                    </div>
                    {tx.success && (
                      <div className="mt-2 flex gap-3 text-xs">
                        <span className="text-zcash-green">
                          {tx.confirmations} confs
                        </span>
                        {tx.consensus_reached && (
                          <span className="text-blue-400">
                            {((tx.agreement_ratio || 0) * 100).toFixed(0)}% consensus
                          </span>
                        )}
                      </div>
                    )}
                    {tx.error && (
                      <div className="mt-1 text-xs text-red-400">
                        {tx.error}
                      </div>
                    )}
                  </div>
                  <div className={`w-2 h-2 rounded-full flex-shrink-0 mt-1 ${
                    tx.success ? 'bg-zcash-green' : 'bg-red-500'
                  }`} />
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}