import type { BridgeTransaction } from "../types";

interface TransactionRowProps {
  transaction: BridgeTransaction;
  onSelect: (tx: BridgeTransaction) => void;
}

function getStatusStyle(status: BridgeTransaction["status"]): string {
  switch (status) {
    case "completed":
      return "bg-green-500/20 text-green-400";
    case "pending":
      return "bg-yellow-500/20 text-yellow-400";
    case "failed":
      return "bg-red-500/20 text-red-400";
  }
}

function getStatusIcon(status: BridgeTransaction["status"]): string {
  switch (status) {
    case "completed":
      return "\u2713";
    case "pending":
      return "\u25CB";
    case "failed":
      return "\u2717";
  }
}

function formatRelativeTime(dateString: string): string {
  const now = Date.now();
  const then = new Date(dateString).getTime();
  const diffMs = now - then;
  const diffSec = Math.floor(diffMs / 1000);

  if (diffSec < 60) return `${diffSec}s ago`;
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;
  const diffDay = Math.floor(diffHr / 24);
  if (diffDay < 30) return `${diffDay}d ago`;
  return new Date(dateString).toLocaleDateString();
}

function truncateHash(hash: string): string {
  if (hash.length <= 14) return hash;
  return `${hash.slice(0, 6)}...${hash.slice(-6)}`;
}

function formatAmount(amount: number, asset: string): string {
  return `${amount.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 6,
  })} ${asset}`;
}

export default function TransactionRow({
  transaction: tx,
  onSelect,
}: TransactionRowProps) {
  return (
    <tr
      onClick={() => onSelect(tx)}
      className="border-b border-stellar-border hover:bg-stellar-border/30 cursor-pointer transition-colors"
    >
      {/* Status */}
      <td className="px-4 py-3">
        <span
          className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusStyle(
            tx.status
          )}`}
        >
          <span aria-hidden="true">{getStatusIcon(tx.status)}</span>
          {tx.status.charAt(0).toUpperCase() + tx.status.slice(1)}
        </span>
      </td>

      {/* Tx Hash */}
      <td className="px-4 py-3">
        <span className="text-sm text-stellar-blue font-mono">
          {truncateHash(tx.txHash)}
        </span>
      </td>

      {/* Bridge */}
      <td className="px-4 py-3 text-sm text-white">{tx.bridge}</td>

      {/* Asset + Amount */}
      <td className="px-4 py-3 text-sm text-white font-medium">
        {formatAmount(tx.amount, tx.asset)}
      </td>

      {/* Direction */}
      <td className="px-4 py-3">
        <div className="flex items-center gap-1 text-xs text-stellar-text-secondary">
          <span>{tx.sourceChain}</span>
          <span className="text-stellar-blue">&rarr;</span>
          <span>{tx.destinationChain}</span>
        </div>
      </td>

      {/* Time */}
      <td className="px-4 py-3">
        <span
          className="text-sm text-stellar-text-secondary"
          title={new Date(tx.timestamp).toLocaleString()}
        >
          {formatRelativeTime(tx.timestamp)}
        </span>
      </td>
    </tr>
  );
}

export function TransactionRowSkeleton() {
  return (
    <tr className="border-b border-stellar-border">
      {[1, 2, 3, 4, 5, 6].map((i) => (
        <td key={i} className="px-4 py-3">
          <div className="h-4 bg-stellar-border rounded animate-pulse w-20" />
        </td>
      ))}
    </tr>
  );
}

export function TransactionCard({
  transaction: tx,
  onSelect,
}: TransactionRowProps) {
  return (
    <button
      type="button"
      onClick={() => onSelect(tx)}
      className="w-full text-left bg-stellar-card border border-stellar-border rounded-lg p-4 hover:border-stellar-blue transition-colors space-y-3"
    >
      <div className="flex items-center justify-between">
        <span
          className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusStyle(
            tx.status
          )}`}
        >
          <span aria-hidden="true">{getStatusIcon(tx.status)}</span>
          {tx.status.charAt(0).toUpperCase() + tx.status.slice(1)}
        </span>
        <span
          className="text-xs text-stellar-text-secondary"
          title={new Date(tx.timestamp).toLocaleString()}
        >
          {formatRelativeTime(tx.timestamp)}
        </span>
      </div>

      <div className="flex items-center justify-between">
        <span className="text-white font-medium text-sm">
          {formatAmount(tx.amount, tx.asset)}
        </span>
        <span className="text-xs text-stellar-text-secondary">{tx.bridge}</span>
      </div>

      <div className="flex items-center gap-1 text-xs text-stellar-text-secondary">
        <span>{tx.sourceChain}</span>
        <span className="text-stellar-blue">&rarr;</span>
        <span>{tx.destinationChain}</span>
      </div>

      <div className="text-xs text-stellar-blue font-mono">
        {truncateHash(tx.txHash)}
      </div>
    </button>
  );
}
