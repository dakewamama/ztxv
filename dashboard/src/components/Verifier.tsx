import { useState } from 'react'

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000'

interface VerificationResult {
  tx_hash: string
  valid: boolean
  confidence: number
  details: {
    network_propagation: number
    estimated_finality_seconds: number
    proof_valid: boolean
    nullifier_unused: boolean
  }
  status: {
    type: string
    confirmations?: number
  }
  timestamp: number
}

export default function Verifier() {
  const [txHash, setTxHash] = useState('')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<VerificationResult | null>(null)
  const [error, setError] = useState<string | null>(null)

  const handleVerify = async () => {
    if (!txHash.trim()) {
      setError('Enter a transaction hash')
      return
    }

    setLoading(true)
    setError(null)
    setResult(null)

    try {
      const response = await fetch(`${API_URL}/verify/${txHash}`)
      const data = await response.json()

      if (data.success) {
        setResult(data.data)
      } else {
        setError(data.error || 'Verification failed')
      }
    } catch (err) {
      setError(`Connection failed: ${err instanceof Error ? err.message : 'Unknown error'}`)
    } finally {
      setLoading(false)
    }
  }

  const useExample = () => {
    setTxHash('0d19877eb803b3806f640d2eeb89572ca8d2f68a26f1b6e1b1cccc6b6e9e0e2e')
  }

  return (
    <div className="max-w-5xl mx-auto px-4 py-16">
      {/* Glass Card Container */}
      <div className="glass-card p-8">
        <h2 className="text-4xl font-bold mb-8">
          <span className="text-zcash-green">Live</span> Verification
        </h2>

        {/* Input Section */}
        <div className="mb-8 space-y-4">
          <div className="flex gap-3">
            <input
              type="text"
              value={txHash}
              onChange={(e) => setTxHash(e.target.value)}
              placeholder="Enter Zcash transaction hash..."
              className="flex-1 glass-input"
              onKeyDown={(e) => e.key === 'Enter' && handleVerify()}
            />
            <button
              onClick={useExample}
              className="glass-button px-8"
            >
              Example
            </button>
          </div>

          <button
            onClick={handleVerify}
            disabled={loading}
            className="w-full glass-button-primary"
          >
            {loading ? (
              <span className="flex items-center justify-center gap-2">
                <span className="loading-spinner" />
                Verifying...
              </span>
            ) : (
              'Verify Transaction'
            )}
          </button>
        </div>

        {/* Error */}
        {error && (
          <div className="glass-error">
            <span className="text-lg">⚠️</span>
            <p>{error}</p>
          </div>
        )}

        {/* Result */}
        {result && (
          <div className="glass-result fade-in">
            {/* Header */}
            <div className="mb-8">
              <div className="flex items-center gap-4 mb-3">
                <div className={`status-indicator ${result.valid ? 'status-valid' : 'status-invalid'}`} />
                <h3 className="text-3xl font-bold">
                  {result.valid ? 'Valid Transaction' : 'Invalid Transaction'}
                </h3>
              </div>
              <p className="text-gray-400 font-mono text-sm break-all">
                {result.tx_hash}
              </p>
            </div>

            {/* Stats Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
              <div className="stat-card">
                <div className="stat-label">Confidence</div>
                <div className="stat-value text-zcash-green">
                  {result.confidence.toFixed(1)}%
                </div>
                <div className="stat-bar">
                  <div 
                    className="stat-bar-fill bg-zcash-green"
                    style={{ width: `${result.confidence}%` }}
                  />
                </div>
              </div>

              <div className="stat-card">
                <div className="stat-label">Network Propagation</div>
                <div className="stat-value">
                  {(result.details.network_propagation * 100).toFixed(0)}%
                </div>
                <div className="stat-bar">
                  <div 
                    className="stat-bar-fill bg-blue-500"
                    style={{ width: `${result.details.network_propagation * 100}%` }}
                  />
                </div>
              </div>

              <div className="stat-card">
                <div className="stat-label">Finality ETA</div>
                <div className="stat-value">
                  {result.details.estimated_finality_seconds}s
                </div>
              </div>

              <div className="stat-card">
                <div className="stat-label">Status</div>
                <div className="stat-value">
                  {result.status.type === 'Confirmed' ? (
                    <span className="text-zcash-green">✓ Confirmed</span>
                  ) : result.status.type === 'InBlock' ? (
                    <span className="text-yellow-400">⏳ {result.status.confirmations} confs</span>
                  ) : (
                    <span className="text-gray-400">⋯ Pending</span>
                  )}
                </div>
              </div>
            </div>

            {/* Details */}
            <div className="glass-details">
              <div className="detail-row">
                <span>Proof Valid</span>
                <span className={result.details.proof_valid ? 'text-zcash-green' : 'text-red-400'}>
                  {result.details.proof_valid ? '✓ Yes' : '✗ No'}
                </span>
              </div>
              <div className="detail-row">
                <span>Nullifier Status</span>
                <span className={result.details.nullifier_unused ? 'text-zcash-green' : 'text-red-400'}>
                  {result.details.nullifier_unused ? '✓ Unused' : '✗ Spent'}
                </span>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
