import TransactionHistory from "../components/TransactionHistory";

export default function Transactions() {
  return (
    <div className="space-y-8">
      <header>
        <h1 className="text-3xl font-bold text-white">Transaction History</h1>
        <p className="mt-2 text-stellar-text-secondary">
          Browse recent bridge transfers with real-time status tracking
        </p>
      </header>

      <TransactionHistory />
    </div>
  );
}
