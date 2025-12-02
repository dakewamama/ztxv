import { useState } from 'react'
import { useScramble } from 'use-scramble'

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000'

interface NodeResponse {
  node_name: string
  confirmations: number
  response_time_ms: number
  success: boolean
}

interface ConsensusResult {
  consensus_reached: boolean
  agreement_ratio: number
  confirmations: number
  responses: NodeResponse[]
  total_time_ms: number
}

export default function ConsensusView() {
  const [txHash, setTxHash] = useState('')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<ConsensusResult | null>(null)

  const { ref: titleRef } = useScramble({
    text: 'Multi-RPC Consensus',
    speed: 0.6,
    tick: 1,
    step: 1,
    scramble: 4,
    seed: 0,
    playOnMount: true,
  })

  const handleVerify = async () => {
    if (!txHash.trim()) return

    setLoading(true)
    setResult(null)

    try {
      const response = await fetch(`${API_URL}/verify-consensus/${txHash}`)
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
    setTxHash('2916ab4fad38bafc0632716ed431d24e91213e23233ce99583962ed82ffed10e')
  }

  return (
    <div className="glass-card p-8 fade-in-up">
      <h2 className="text-3xl font-bold mb-6">
        <span className="text-zcash-green scramble-text" ref={titleRef} />
      </h2>

      <div className="mb-6 space-y-3">
        <div className="flex gap-3">
          <input
            type="text"
            value={txHash}
            onChange={(e) => setTxHash(e.target.value)}
            placeholder="Enter transaction hash..."
            className="flex-1 glass-input font-mono text-sm"
            onKeyDown={(e) => e.key === 'Enter' && handleVerify()}
          />
          <button onClick={useExample} className="glass-button px-6">
            Example
          </button>
        </div>
        <button
          onClick={handleVerify}
          disabled={loading}
          className="w-full glass-button-primary py-3"
        >
          {loading ? 'Verifying...' : 'Verify with Consensus'}
        </button>
      </div>

      {result && (
        <div className="space-y-4 fade-in-scale">
          {/* Consensus Status */}
          <div className={`p-4 rounded-2xl border-2 ${
            result.consensus_reached 
              ? 'bg-zcash-green/10 border-zcash-green/30' 
              : 'bg-red-500/10 border-red-500/30'
          }`}>
            <div className="flex items-center justify-between">
              <span className="font-mono text-sm">Consensus Status</span>
              <span className={`font-bold ${
                result.consensus_reached ? 'text-zcash-green' : 'text-red-500'
              }`}>
                {result.consensus_reached ? '✓ REACHED' : '✗ NOT REACHED'}
              </span>
            </div>
            <div className="mt-2 flex items-center gap-2">
              <div className="flex-1 h-2 bg-black/40 rounded-full overflow-hidden">
                <div 
                  className={`h-full ${
                    result.consensus_reached ? 'bg-zcash-green' : 'bg-red-500'
                  }`}
                  style={{ width: `${result.agreement_ratio * 100}%` }}
                />
              </div>
              <span className="text-sm font-mono">
                {(result.agreement_ratio * 100).toFixed(0)}%
              </span>
            </div>
          </div>

          {/* Node Responses */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {result.responses.map((node, idx) => (
              <div 
                key={idx}
                className={`glass-card p-4 fade-in-delay-${idx + 1}`}
              >
                <div className="flex items-center justify-between mb-3">
                  <span className="font-bold text-sm">{node.node_name}</span>
                  <div className={`w-2 h-2 rounded-full ${
                    node.success ? 'bg-zcash-green' : 'bg-red-500'
                  }`} />
                </div>
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-500">Confirmations</span>
                    <span className="font-mono text-zcash-green">
                      {node.confirmations}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-500">Response Time</span>
                    <span className="font-mono text-blue-400">
                      {node.response_time_ms}ms
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Summary */}
          <div className="glass-card p-4 font-mono text-sm">
            <div className="flex justify-between mb-2">
              <span className="text-gray-500">Total Query Time</span>
              <span className="text-purple-400">{result.total_time_ms}ms</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">Canonical Confirmations</span>
              <span className="text-zcash-green">{result.confirmations}</span>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}