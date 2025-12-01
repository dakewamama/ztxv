import { useState, useEffect, useRef } from 'react'
import { useScramble } from 'use-scramble'

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
  status: 
    | { Confirmed: number }
    | { InBlock: number }
    | { InMempool: null }
    | { NotFound: null }
  timestamp: number
}

export default function Verifier() {
  const [txHash, setTxHash] = useState('')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<VerificationResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const resultRef = useRef<HTMLDivElement>(null)

  // Parse status to get type and confirmations
  const getStatusInfo = (status: VerificationResult['status']) => {
    if ('Confirmed' in status) {
      return { type: 'Confirmed', confirmations: status.Confirmed }
    }
    if ('InBlock' in status) {
      return { type: 'InBlock', confirmations: status.InBlock }
    }
    if ('InMempool' in status) {
      return { type: 'InMempool', confirmations: 0 }
    }
    return { type: 'NotFound', confirmations: 0 }
  }

  const statusInfo = result ? getStatusInfo(result.status) : { type: 'Unknown', confirmations: 0 }

  // Scramble effects - only initialize after we have result
  const { ref: titleRef } = useScramble({
    text: 'Live Verification',
    speed: 0.6,
    tick: 1,
    step: 1,
    scramble: 4,
    seed: 0,
    playOnMount: true,
  })

  const { ref: validRef, replay: replayValid } = useScramble({
    text: result?.valid ? 'VALID' : 'INVALID',
    speed: 0.5,
    tick: 2,
    step: 1,
    scramble: 5,
    seed: 1,
    playOnMount: false,
  })

  const { ref: txHashRef, replay: replayTxHash } = useScramble({
    text: result?.tx_hash || '',
    speed: 1,
    tick: 1,
    step: 2,
    scramble: 2,
    seed: 2,
    playOnMount: false,
  })

  const { ref: confidenceRef, replay: replayConfidence } = useScramble({
    text: result ? `${result.confidence.toFixed(1)}%` : '0.0%',
    speed: 0.8,
    tick: 1,
    step: 1,
    scramble: 3,
    seed: 3,
    playOnMount: false,
  })

  const { ref: propagationRef, replay: replayPropagation } = useScramble({
    text: result ? `${(result.details.network_propagation * 100).toFixed(0)}%` : '0%',
    speed: 0.8,
    tick: 1,
    step: 1,
    scramble: 3,
    seed: 4,
    playOnMount: false,
  })

  const { ref: finalityRef, replay: replayFinality } = useScramble({
    text: result ? `${result.details.estimated_finality_seconds}` : '0',
    speed: 0.6,
    tick: 1,
    step: 1,
    scramble: 2,
    seed: 5,
    playOnMount: false,
  })

  const { ref: confirmationsRef, replay: replayConfirmations } = useScramble({
    text: `${statusInfo.confirmations}`,
    speed: 0.8,
    tick: 1,
    step: 1,
    scramble: 3,
    seed: 7,
    playOnMount: false,
  })

  const { ref: timestampRef, replay: replayTimestamp } = useScramble({
    text: result ? `${result.timestamp}` : '0',
    speed: 0.9,
    tick: 1,
    step: 1,
    scramble: 2,
    seed: 8,
    playOnMount: false,
  })

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
        // Set result FIRST, then trigger animations
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

  // Trigger scramble animations AFTER result is set
  useEffect(() => {
    if (result) {
      // Staggered scramble animations
      setTimeout(() => replayValid(), 200)
      setTimeout(() => replayTxHash(), 400)
      setTimeout(() => replayConfidence(), 600)
      setTimeout(() => replayPropagation(), 800)
      setTimeout(() => replayFinality(), 1000)
      setTimeout(() => replayConfirmations(), 1200)
      setTimeout(() => replayTimestamp(), 1400)
    }
  }, [result, replayValid, replayTxHash, replayConfidence, replayPropagation, replayFinality, replayConfirmations, replayTimestamp])

  useEffect(() => {
    if (result && resultRef.current) {
      setTimeout(() => {
        resultRef.current?.scrollIntoView({ 
          behavior: 'smooth', 
          block: 'center' 
        })
      }, 300)
    }
  }, [result])

  const useExample = () => {
    setTxHash('0d19877eb803b3806f640d2eeb89572ca8d2f68a26f1b6e1b1cccc6b6e9e0e2e')
  }

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
  }

  const getConfidenceLevel = (confidence: number) => {
    if (confidence >= 99) return { label: 'CONFIRMED', color: 'text-zcash-green' }
    if (confidence >= 85) return { label: 'HIGH', color: 'text-blue-400' }
    if (confidence >= 75) return { label: 'MEDIUM', color: 'text-yellow-400' }
    return { label: 'LOW', color: 'text-red-400' }
  }

  const getPropagationLevel = (propagation: number) => {
    if (propagation >= 0.95) return { label: 'FULL NETWORK', color: 'text-zcash-green' }
    if (propagation >= 0.7) return { label: 'MAJORITY', color: 'text-blue-400' }
    if (propagation >= 0.5) return { label: 'PARTIAL', color: 'text-yellow-400' }
    return { label: 'LIMITED', color: 'text-red-400' }
  }

  const renderJsonValue = (value: unknown) => {
    if (typeof value === 'boolean') {
      return (
        <span className={value ? 'json-boolean-true' : 'json-boolean-false'}>
          {value.toString()}
        </span>
      )
    }

    if (typeof value === 'number') {
      return <span className="json-number">{value}</span>
    }

    if (typeof value === 'string') {
      return <span className="json-string">"{value}"</span>
    }

    return <span>{String(value)}</span>
  }

  return (
    <div className="max-w-5xl mx-auto px-4 py-16">
      <div className="glass-card p-8 fade-in-up">
        <h2 className="text-4xl font-bold mb-8">
          <span className="text-zcash-green scramble-text" ref={titleRef} />
        </h2>

        <div className="mb-8 space-y-4">
          <div className="flex gap-3">
            <input
              type="text"
              value={txHash}
              onChange={(e) => setTxHash(e.target.value)}
              placeholder="Enter Zcash transaction hash..."
              className="flex-1 glass-input font-mono"
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
                <span className="scramble-text">Verifying...</span>
              </span>
            ) : (
              'Verify Transaction'
            )}
          </button>
        </div>

        {error && (
          <div className="glass-error fade-in">
            <span className="text-lg">error: </span>
            <p>{error}</p>
          </div>
        )}

        {result && (
          <div ref={resultRef} className="json-container fade-in-scale">
            {/* JSON Header */}
            <div className="json-header">
              <span className="json-bracket">{'{'}</span>
              <div className="json-status-badge">
                <span className={result.valid ? 'status-dot-valid' : 'status-dot-invalid'} />
                <span className="text-xs font-mono scramble-text" ref={validRef} />
              </div>
            </div>

            {/* JSON Body */}
            <div className="json-body">
              {/* tx_hash */}
              <div className="json-line fade-in-delay-1">
                <span className="json-key">"tx_hash"</span>
                <span className="json-colon">:</span>
                <span className="json-string break-all">"<span className="scramble-text" ref={txHashRef} />"</span>
                <span className="json-comma">,</span>
              </div>

              {/* valid */}
              <div className="json-line fade-in-delay-1">
                <span className="json-key">"valid"</span>
                <span className="json-colon">:</span>
                {renderJsonValue(result.valid)}
                <span className="json-comma">,</span>
              </div>

              {/* confidence with enhanced bar */}
              <div className="json-line fade-in-delay-2">
                <span className="json-key">"confidence"</span>
                <span className="json-colon">:</span>
                <span className="json-number scramble-text" ref={confidenceRef} />
                <span className="json-comma">,</span>
                
                {/* Enhanced Progress Bar */}
                <div className="progress-bar-container">
                  <div className="progress-bar-header">
                    <span className={`progress-label ${getConfidenceLevel(result.confidence).color}`}>
                      {getConfidenceLevel(result.confidence).label}
                    </span>
                    <span className="progress-percentage">
                      {result.confidence.toFixed(1)}%
                    </span>
                  </div>
                  <div className="progress-bar-track">
                    <div 
                      className="progress-bar-fill progress-bar-green"
                      style={{ width: `${result.confidence}%` }}
                    />
                    <div className="progress-threshold" style={{ left: '75%' }}>
                      <span className="progress-threshold-label">75</span>
                    </div>
                    <div className="progress-threshold" style={{ left: '85%' }}>
                      <span className="progress-threshold-label">85</span>
                    </div>
                    <div className="progress-threshold" style={{ left: '99%' }}>
                      <span className="progress-threshold-label">99</span>
                    </div>
                  </div>
                </div>
              </div>

              {/* details object */}
              <div className="json-line fade-in-delay-3">
                <span className="json-key">"details"</span>
                <span className="json-colon">:</span>
                <span className="json-bracket">{'{'}</span>
              </div>

              <div className="json-nested">
                <div className="json-line">
                  <span className="json-key">"proof_valid"</span>
                  <span className="json-colon">:</span>
                  {renderJsonValue(result.details.proof_valid)}
                  <span className="json-comma">,</span>
                </div>

                <div className="json-line">
                  <span className="json-key">"nullifier_unused"</span>
                  <span className="json-colon">:</span>
                  {renderJsonValue(result.details.nullifier_unused)}
                  <span className="json-comma">,</span>
                </div>

                {/* network_propagation with enhanced bar */}
                <div className="json-line">
                  <span className="json-key">"network_propagation"</span>
                  <span className="json-colon">:</span>
                  <span className="json-number scramble-text" ref={propagationRef} />
                  <span className="json-comma">,</span>
                  
                  {/* Enhanced Progress Bar */}
                  <div className="progress-bar-container">
                    <div className="progress-bar-header">
                      <span className={`progress-label ${getPropagationLevel(result.details.network_propagation).color}`}>
                        {getPropagationLevel(result.details.network_propagation).label}
                      </span>
                      <span className="progress-percentage">
                        {(result.details.network_propagation * 100).toFixed(0)}%
                      </span>
                    </div>
                    <div className="progress-bar-track">
                      <div 
                        className="progress-bar-fill progress-bar-blue"
                        style={{ width: `${result.details.network_propagation * 100}%` }}
                      />
                      <div className="progress-threshold" style={{ left: '50%' }}>
                        <span className="progress-threshold-label">50</span>
                      </div>
                      <div className="progress-threshold" style={{ left: '70%' }}>
                        <span className="progress-threshold-label">70</span>
                      </div>
                      <div className="progress-threshold" style={{ left: '95%' }}>
                        <span className="progress-threshold-label">95</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="json-line">
                  <span className="json-key">"estimated_finality_seconds"</span>
                  <span className="json-colon">:</span>
                  <span className="json-number scramble-text" ref={finalityRef} />
                </div>
              </div>

              <div className="json-line fade-in-delay-3">
                <span className="json-bracket">{'}'}</span>
                <span className="json-comma">,</span>
              </div>

              {/* status object */}
              <div className="json-line fade-in-delay-4">
                <span className="json-key">"status"</span>
                <span className="json-colon">:</span>
                <span className="json-bracket">{'{'}</span>
              </div>

              <div className="json-nested">
                <div className="json-line">
                  <span className={
                    statusInfo.type === 'Confirmed' ? 'json-key text-zcash-green' :
                    statusInfo.type === 'InBlock' ? 'json-key text-yellow-400' :
                    'json-key'
                  }>"{statusInfo.type}"</span>
                  <span className="json-colon">:</span>
                  <span className="json-number scramble-text" ref={confirmationsRef} />
                </div>
              </div>

              <div className="json-line fade-in-delay-4">
                <span className="json-bracket">{'}'}</span>
                <span className="json-comma">,</span>
              </div>

              {/* timestamp */}
              <div className="json-line fade-in-delay-5">
                <span className="json-key">"timestamp"</span>
                <span className="json-colon">:</span>
                <span className="json-number scramble-text" ref={timestampRef} />
                <span className="json-comment"> // {formatTimestamp(result.timestamp)}</span>
              </div>
            </div>

            {/* JSON Footer */}
            <div className="json-footer">
              <span className="json-bracket">{'}'}</span>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
