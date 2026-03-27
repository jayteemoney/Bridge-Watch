export interface Asset {
  symbol: string;
  name: string;
}

export interface HealthFactors {
  liquidityDepth: number;
  priceStability: number;
  bridgeUptime: number;
  reserveBacking: number;
  volumeTrend: number;
}

export interface HealthScore {
  symbol: string;
  overallScore: number;
  factors: HealthFactors;
  trend: "improving" | "stable" | "deteriorating";
  lastUpdated: string;
}

export type HealthStatus = "healthy" | "warning" | "critical";

export interface AssetWithHealth extends Asset {
  health: HealthScore | null;
}

export type SortField = "symbol" | "score";
export type SortOrder = "asc" | "desc";
export type FilterStatus = "all" | HealthStatus;

// Transaction History types
export type TransactionStatus = "pending" | "completed" | "failed";

export interface BridgeTransaction {
  id: string;
  txHash: string;
  bridge: string;
  asset: string;
  amount: number;
  sourceChain: string;
  destinationChain: string;
  senderAddress: string;
  recipientAddress: string;
  status: TransactionStatus;
  fee: number;
  timestamp: string;
  confirmedAt: string | null;
  stellarTxHash: string | null;
  ethereumTxHash: string | null;
  blockNumber: number | null;
}

export interface TransactionFilters {
  bridge: string;
  asset: string;
  status: TransactionStatus | "all";
  search: string;
  dateFrom: string;
  dateTo: string;
}

export interface TransactionPage {
  transactions: BridgeTransaction[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}
