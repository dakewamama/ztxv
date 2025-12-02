import Verifier from './components/Verifier'
import MempoolFeed from './components/MempoolFeed'
import ConsensusView from './components/ConsensusView'
import BatchVerifier from './components/BatchVerifier'

function App() {
  return (
    <div className="min-h-screen bg-deep-black text-white">
      {/* Hero */}
      <div className="max-w-6xl mx-auto px-4 py-20 fade-in">
        <h1 className="text-6xl font-bold mb-4">
          <span className="text-zcash-green">ztxv</span>
        </h1>
        <p className="text-2xl text-gray-400 mb-2">
          Real-time Zcash Transaction Verification
        </p>
        <p className="text-xl text-gray-500 mb-6">
          Multi-RPC consensus • Mempool monitoring • Batch processing
        </p>
        
        {/* Stats */}
        <div className="grid grid-cols-3 gap-4 mt-8 max-w-2xl">
          <div className="glass-card p-4 text-center">
            <div className="text-3xl font-bold text-zcash-green mb-1">24x</div>
            <div className="text-sm text-gray-500">Faster</div>
          </div>
          <div className="glass-card p-4 text-center">
            <div className="text-3xl font-bold text-blue-400 mb-1">&lt;2s</div>
            <div className="text-sm text-gray-500">Verification</div>
          </div>
          <div className="glass-card p-4 text-center">
            <div className="text-3xl font-bold text-purple-400 mb-1">4/4</div>
            <div className="text-sm text-gray-500">RPC Nodes</div>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-6xl mx-auto px-4 pb-20 space-y-8">
        {/* 1. Live Verification - FIRST */}
        <Verifier />

        {/* 2. Two Column Layout - Consensus + Batch */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          <ConsensusView />
          <BatchVerifier />
        </div>

        {/* 3. Mempool Feed - LAST */}
        <MempoolFeed />
      </div>

      {/* Footer */}
      <div className="max-w-6xl mx-auto px-4 pb-12 text-center text-gray-600">
        <p className="text-sm">
          ztxv... The verification layer • powered by StauroX
        </p>
      </div>
    </div>
  )
}

export default App