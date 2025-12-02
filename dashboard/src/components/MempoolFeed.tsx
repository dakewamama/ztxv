import { useState, useEffect, useRef } from 'react'
import { useScramble } from 'use-scramble'

const WS_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:3000'

interface MempoolTx {
  tx_hash: string
  detected_at: number
  initial_confidence: number
}

export default function MempoolFeed() {
  const [txs, setTxs] = useState<MempoolTx[]>([])
  const [isConnected, setIsConnected] = useState(false)
  const [newTxIndex, setNewTxIndex] = useState<number | null>(null)
  const wsRef = useRef<WebSocket | null>(null)

  const { ref: titleRef } = useScramble({
    text: 'Live Mempool Feed',
    speed: 0.6,
    tick: 1,
    step: 1,
    scramble: 4,
    seed: 0,
    playOnMount: true,
  })

  useEffect(() => {
    const ws = new WebSocket(`${WS_URL}/ws`)
    wsRef.current = ws

    ws.onopen = () => {
      setIsConnected(true)
      console.log('Mempool feed connected')
    }

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        if (data.type === 'mempool' && data.data) {
          setTxs((prev) => {
            const newTxs = [data.data, ...prev].slice(0, 3)
            setNewTxIndex(0)
            setTimeout(() => setNewTxIndex(null), 2000)
            return newTxs
          })
        }
      } catch (err) {
        console.error('WebSocket message error:', err)
      }
    }

    ws.onerror = () => setIsConnected(false)
    ws.onclose = () => setIsConnected(false)

    return () => {
      ws.close()
    }
  }, [])

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000)
    return date.toLocaleTimeString('en-US', { 
      hour: '2-digit', 
      minute: '2-digit', 
      second: '2-digit' 
    })
  }

  return (
    <div className="glass-card p-8 fade-in-up">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-3xl font-bold">
          <span className="text-zcash-green scramble-text" ref={titleRef} />
        </h2>
        <div className="flex items-center gap-2">
          <div className={`relative w-3 h-3 rounded-full ${isConnected ? 'bg-zcash-green' : 'bg-red-500'}`}>
            {isConnected && (
              <div className="absolute inset-0 w-3 h-3 rounded-full bg-zcash-green animate-ping" />
            )}
          </div>
          <span className="text-sm text-gray-400 font-mono">
            {isConnected ? 'LIVE' : 'DISCONNECTED'}
          </span>
        </div>
      </div>

      <div className="space-y-3 max-h-[400px] overflow-y-auto pr-2 custom-scrollbar">
        {txs.length === 0 ? (
          <div className="text-center py-12 text-gray-500">
            <div className="text-6xl mb-4">tx</div>
            <p className="text-lg">Listening for new transactions...</p>
            <p className="text-sm mt-2">Transactions appear here as they hit the mempool</p>
          </div>
        ) : (
          txs.map((tx, idx) => (
            <div
              key={`${tx.tx_hash}-${tx.detected_at}`}
              className={`glass-card p-4 hover:border-zcash-green/30 transition-all cursor-pointer ${
                newTxIndex === idx ? 'new-tx-animation' : 'fade-in-scale'
              }`}
              onClick={() => navigator.clipboard.writeText(tx.tx_hash)}
            >
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs text-gray-500 font-mono">
                  {formatTime(tx.detected_at)}
                </span>
                <div className="flex items-center gap-2">
                  {newTxIndex === idx && (
                    <span className="text-xs px-2 py-1 rounded-full bg-zcash-green/20 text-zcash-green border border-zcash-green/30 animate-pulse">
                      NEW
                    </span>
                  )}
                  <span className="text-xs px-2 py-1 rounded-full bg-yellow-500/10 text-yellow-400 border border-yellow-500/30">
                    0 CONFS
                  </span>
                </div>
              </div>
              <div className={`font-mono text-sm text-gray-300 break-all ${
                newTxIndex === idx ? 'scramble-animation' : ''
              }`}>
                {tx.tx_hash}
              </div>
              <div className="mt-2 flex items-center gap-2">
                <div className="flex-1 h-1 bg-gray-800 rounded-full overflow-hidden">
                  <div 
                    className="h-full bg-yellow-400 transition-all duration-500"
                    style={{ width: `${tx.initial_confidence * 100}%` }}
                  />
                </div>
                <span className="text-xs text-yellow-400 font-mono">
                  {(tx.initial_confidence * 100).toFixed(0)}%
                </span>
              </div>
            </div>
          ))
        )}
      </div>

      {txs.length > 0 && (
        <div className="mt-6 text-center text-sm text-gray-500">
          <p>Showing last 3 transactions • Click to copy hash</p>
        </div>
      )}
    </div>
  )
}