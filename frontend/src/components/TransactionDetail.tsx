import type { BridgeTransaction } from "../types";

interface TransactionDetailProps {
  transaction: BridgeTransaction;
  onClose: () => void;
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

function getStellarExplorerUrl(hash: string): string {
  return `https://stellar.expert/explorer/public/tx/${hash}`;
}

function getEthereumExplorerUrl(hash: string): string {
  return `https://etherscan.io/tx/${hash}`;
}

function truncateAddress(address: string): string {
  if (address.length <= 16) return address;
  return `${address.slice(0, 8)}...${address.slice(-8)}`;
}

function DetailRow({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col sm:flex-row sm:justify-between gap-1 py-2.5 border-b border-stellar-border last:border-b-0">
      <span className="text-sm text-stellar-text-secondary">{label}</span>
      <span className="text-sm text-white break-all sm:text-right">
        {children}
      </span>
    </div>
  );
}

export default function TransactionDetail({
  transaction: tx,
  onClose,
}: TransactionDetailProps) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      role="dialog"
      aria-modal="true"
      aria-label="Transaction details"
    >
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Panel */}
      <div className="relative w-full max-w-lg max-h-[90vh] overflow-y-auto bg-stellar-card border border-stellar-border rounded-xl shadow-2xl">
        {/* Header */}
        <div className="sticky top-0 bg-stellar-card border-b border-stellar-border px-6 py-4 flex items-center justify-between rounded-t-xl">
          <h2 className="text-lg font-semibold text-white">
            Transaction Details
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="text-stellar-text-secondary hover:text-white transition-colors p-1"
            aria-label="Close"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-5 w-5"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fillRule="evenodd"
                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                clipRule="evenodd"
              />
            </svg>
          </button>
        </div>

        {/* Body */}
        <div className="px-6 py-4 space-y-1">
          <DetailRow label="Status">
            <span
              className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusStyle(
                tx.status
              )}`}
            >
              <span aria-hidden="true">{getStatusIcon(tx.status)}</span>
              {tx.status.charAt(0).toUpperCase() + tx.status.slice(1)}
            </span>
          </DetailRow>

          <DetailRow label="Transaction Hash">
            <span className="font-mono text-stellar-blue text-xs">
              {tx.txHash}
            </span>
          </DetailRow>

          <DetailRow label="Bridge">{tx.bridge}</DetailRow>

          <DetailRow label="Asset">{tx.asset}</DetailRow>

          <DetailRow label="Amount">
            {tx.amount.toLocaleString(undefined, {
              minimumFractionDigits: 2,
              maximumFractionDigits: 6,
            })}{" "}
            {tx.asset}
          </DetailRow>

          <DetailRow label="Fee">
            {tx.fee.toLocaleString(undefined, {
              minimumFractionDigits: 2,
              maximumFractionDigits: 6,
            })}{" "}
            {tx.asset}
          </DetailRow>

          <DetailRow label="Source Chain">{tx.sourceChain}</DetailRow>

          <DetailRow label="Destination Chain">
            {tx.destinationChain}
          </DetailRow>

          <DetailRow label="Sender">
            <span className="font-mono text-xs" title={tx.senderAddress}>
              {truncateAddress(tx.senderAddress)}
            </span>
          </DetailRow>

          <DetailRow label="Recipient">
            <span className="font-mono text-xs" title={tx.recipientAddress}>
              {truncateAddress(tx.recipientAddress)}
            </span>
          </DetailRow>

          <DetailRow label="Submitted">
            {new Date(tx.timestamp).toLocaleString()}
          </DetailRow>

          {tx.confirmedAt && (
            <DetailRow label="Confirmed">
              {new Date(tx.confirmedAt).toLocaleString()}
            </DetailRow>
          )}

          {tx.blockNumber !== null && (
            <DetailRow label="Block Number">
              {tx.blockNumber.toLocaleString()}
            </DetailRow>
          )}
        </div>

        {/* Explorer links */}
        <div className="px-6 py-4 border-t border-stellar-border flex flex-wrap gap-3">
          {tx.stellarTxHash && (
            <a
              href={getStellarExplorerUrl(tx.stellarTxHash)}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1.5 bg-stellar-blue/10 text-stellar-blue border border-stellar-blue/30 rounded-lg px-4 py-2 text-sm hover:bg-stellar-blue/20 transition-colors"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-4 w-4"
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" />
                <path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" />
              </svg>
              View on Stellar Expert
            </a>
          )}
          {tx.ethereumTxHash && (
            <a
              href={getEthereumExplorerUrl(tx.ethereumTxHash)}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1.5 bg-purple-500/10 text-purple-400 border border-purple-500/30 rounded-lg px-4 py-2 text-sm hover:bg-purple-500/20 transition-colors"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-4 w-4"
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" />
                <path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" />
              </svg>
              View on Etherscan
            </a>
          )}
        </div>
      </div>
    </div>
  );
}
