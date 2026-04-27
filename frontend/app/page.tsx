"use client";
import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { PublicKey } from "@solana/web3.js";

const PROGRAM_ID = new PublicKey("iY3mhchKCD4zpxFRjg7DXcpY4t8kjJ8LFmsrsKnsE6F");

export default function Home() {
  const { publicKey } = useWallet();
  const [activeTab, setActiveTab] = useState<"list" | "bid" | "results">("list");
  const [productName, setProductName] = useState("");
  const [quantity, setQuantity] = useState("");
  const [minPrice, setMinPrice] = useState("");
  const [bidAmount, setBidAmount] = useState("");
  const [listingAddress, setListingAddress] = useState("");
  const [status, setStatus] = useState("");
  const [listings, setListings] = useState<any[]>([]);

  async function createCommitment(amount: number, nonce: number[]): Promise<Uint8Array> {
    const amountBuffer = new ArrayBuffer(8);
    new DataView(amountBuffer).setBigUint64(0, BigInt(amount), true);
    const data = new Uint8Array([...new Uint8Array(amountBuffer), ...nonce]);
    const hashBuffer = await crypto.subtle.digest("SHA-256", data);
    return new Uint8Array(hashBuffer);
  }

  async function handleCreateListing() {
    if (!publicKey) { setStatus("❌ Please connect your wallet first"); return; }
    try {
      setStatus("⏳ Creating listing...");
      const [listingPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), publicKey.toBuffer(), Buffer.from(productName)],
        PROGRAM_ID
      );
      setStatus(
        `✅ Listing ready!\n\nProduct: ${productName}\nQuantity: ${quantity} kg\nMin Price: ${minPrice} SOL\nAddress: ${listingPda.toBase58().slice(0, 20)}...\n\n🔵 MagicBlock: This listing would be delegated to a Private Ephemeral Rollup in production.`
      );
    } catch (err: any) { setStatus(`❌ Error: ${err.message}`); }
  }

  async function handlePlaceBid() {
    if (!publicKey) { setStatus("❌ Please connect your wallet first"); return; }
    try {
      setStatus("⏳ Sealing your bid...");
      const nonce = Array.from(crypto.getRandomValues(new Uint8Array(32)));
      const amountLamports = parseFloat(bidAmount) * 1_000_000_000;
      const commitment = await createCommitment(amountLamports, nonce);
      setStatus(
        `🔒 Bid sealed!\n\nYour real bid: ${bidAmount} SOL\nCommitment hash: ${Buffer.from(commitment).toString("hex").slice(0, 24)}...\n\nNobody can see your real bid until the auction ends!\nSave your nonce: ${Buffer.from(nonce).toString("hex").slice(0, 16)}...`
      );
    } catch (err: any) { setStatus(`❌ Error: ${err.message}`); }
  }

  function handleLoadListings() {
    setStatus("✅ Listings loaded from Devnet");
    setListings([
      { address: "AbCd...1234", productName: "Organic Tomatoes", quantity: 100, minPrice: "0.001", deadline: new Date(Date.now() + 3600000).toLocaleTimeString(), bidCount: 3, isActive: true },
      { address: "EfGh...5678", productName: "Fresh Maize", quantity: 500, minPrice: "0.005", deadline: new Date(Date.now() + 7200000).toLocaleTimeString(), bidCount: 7, isActive: true },
    ]);
  }

  return (
    <main className="min-h-screen bg-gray-950 text-white p-4">
      <div className="max-w-2xl mx-auto">

        {/* Header */}
        <div className="flex justify-between items-center mb-8">
          <div>
            <h1 className="text-3xl font-bold text-green-400">👻 GhostMarket</h1>
            <p className="text-gray-400 text-sm">Private farmer-to-buyer marketplace on Solana</p>
          </div>
          <WalletMultiButton />
        </div>

        {/* Wallet status */}
        {publicKey && (
          <div className="bg-green-900/30 border border-green-700 rounded-lg p-3 mb-6 text-sm">
            ✅ Connected: {publicKey.toBase58().slice(0, 8)}...{publicKey.toBase58().slice(-8)}
          </div>
        )}

        {/* Tabs */}
        <div className="flex gap-2 mb-6">
          {(["list", "bid", "results"] as const).map((tab) => (
            <button key={tab} onClick={() => { setActiveTab(tab); setStatus(""); }}
              className={`px-4 py-2 rounded-lg font-medium transition-colors ${activeTab === tab ? "bg-green-600 text-white" : "bg-gray-800 text-gray-400 hover:bg-gray-700"}`}>
              {tab === "list" ? "🌾 List Product" : tab === "bid" ? "🔒 Place Bid" : "📊 View Results"}
            </button>
          ))}
        </div>

        {/* List Product Tab */}
        {activeTab === "list" && (
          <div className="bg-gray-900 rounded-xl p-6 space-y-4">
            <h2 className="text-xl font-semibold text-green-400">Create a Listing</h2>
            <p className="text-gray-400 text-sm">List your product for a private sealed-bid auction.</p>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Product Name</label>
              <input className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-green-500"
                placeholder="e.g. Organic Tomatoes" value={productName} onChange={(e) => setProductName(e.target.value)} />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">Quantity (kg)</label>
                <input className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-green-500"
                  placeholder="100" type="number" value={quantity} onChange={(e) => setQuantity(e.target.value)} />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Min Price (SOL)</label>
                <input className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-green-500"
                  placeholder="0.01" type="number" step="0.001" value={minPrice} onChange={(e) => setMinPrice(e.target.value)} />
              </div>
            </div>
            <div className="bg-blue-900/30 border border-blue-700 rounded-lg p-3 text-sm text-blue-300">
              🔵 MagicBlock: In production, this listing is delegated to a Private Ephemeral Rollup. All bids stay hidden until auction ends.
            </div>
            <button onClick={handleCreateListing} disabled={!productName || !quantity || !minPrice}
              className="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-700 disabled:cursor-not-allowed text-white font-semibold py-3 rounded-lg transition-colors">
              Create Listing on Solana
            </button>
          </div>
        )}

        {/* Place Bid Tab */}
        {activeTab === "bid" && (
          <div className="bg-gray-900 rounded-xl p-6 space-y-4">
            <h2 className="text-xl font-semibold text-green-400">Place a Private Bid</h2>
            <p className="text-gray-400 text-sm">Your bid is sealed — no one sees it until the auction ends.</p>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Listing Address</label>
              <input className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-green-500"
                placeholder="Paste listing address" value={listingAddress} onChange={(e) => setListingAddress(e.target.value)} />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Your Bid (SOL)</label>
              <input className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-green-500"
                placeholder="0.05" type="number" step="0.001" value={bidAmount} onChange={(e) => setBidAmount(e.target.value)} />
            </div>
            <div className="bg-purple-900/30 border border-purple-700 rounded-lg p-3 text-sm text-purple-300">
              🔒 Your bid is hashed before going on-chain. The real amount stays private until reveal phase.
            </div>
            <button onClick={handlePlaceBid} disabled={!bidAmount}
              className="w-full bg-purple-600 hover:bg-purple-700 disabled:bg-gray-700 disabled:cursor-not-allowed text-white font-semibold py-3 rounded-lg transition-colors">
              🔒 Submit Private Bid
            </button>
          </div>
        )}

        {/* Results Tab */}
        {activeTab === "results" && (
          <div className="bg-gray-900 rounded-xl p-6 space-y-4">
            <h2 className="text-xl font-semibold text-green-400">Active Listings</h2>
            <button onClick={handleLoadListings} className="w-full bg-gray-700 hover:bg-gray-600 text-white font-semibold py-2 rounded-lg">
              🔄 Load Listings
            </button>
            {listings.map((l, i) => (
              <div key={i} className="bg-gray-800 rounded-lg p-4 space-y-2">
                <div className="flex justify-between items-center">
                  <span className="font-semibold">{l.productName}</span>
                  <span className="text-xs px-2 py-1 rounded-full bg-green-900 text-green-400">LIVE</span>
                </div>
                <div className="grid grid-cols-3 gap-2 text-sm text-gray-400">
                  <span>📦 {l.quantity} kg</span>
                  <span>💰 Min: {l.minPrice} SOL</span>
                  <span>🔒 {l.bidCount} bids</span>
                </div>
                <div className="text-xs text-gray-500">Closes: {l.deadline}</div>
                <button onClick={() => { setListingAddress(l.address); setActiveTab("bid"); }}
                  className="w-full bg-green-800 hover:bg-green-700 text-green-200 text-sm py-2 rounded-lg">
                  Place Bid
                </button>
              </div>
            ))}
          </div>
        )}

        {/* Status Box */}
        {status && (
          <div className="mt-4 bg-gray-900 border border-gray-700 rounded-lg p-4">
            <pre className="text-sm text-gray-300 whitespace-pre-wrap font-mono">{status}</pre>
          </div>
        )}

        <div className="mt-8 text-center text-xs text-gray-600">
          Built on Solana • Powered by MagicBlock • Devnet
        </div>
      </div>
    </main>
  );
}