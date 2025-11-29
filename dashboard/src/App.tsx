import Verifier from './components/Verifier'

function App() {
  return (
    <div className="min-h-screen bg-deep-black text-white">
      <div className="max-w-6xl mx-auto px-4 py-20 fade-in">
        <h1 className="text-6xl font-bold mb-4">
          <span className="text-zcash-green">ztxv</span>
        </h1>
        <p className="text-2xl text-gray-400 mb-2">
          Real-time Zcash Transaction Verification
        </p>
        <p className="text-xl text-gray-500">
          10x faster than traditional bridge verification
        </p>
      </div>

      {/* Verifier Component */}
      <Verifier />
    </div>
  )
}

export default App
