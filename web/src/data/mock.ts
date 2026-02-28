export type Asset = {
  id: string;
  name: string;
  symbol: string;
  amount: number;
  valueUsd: number;
  custody: boolean;
  status: "active" | "pending" | "locked";
};

export type Beneficiary = {
  id: string;
  name: string;
  address: string;
  percentage: number;
  status: "verified" | "pending";
};

export type Activity = {
  id: string;
  title: string;
  detail: string;
  timestamp: string;
  type: "verification" | "deposit" | "distribution" | "alert";
};

export const seedAssets: Asset[] = [
  {
    id: "asset-1",
    name: "Stellar Lumens",
    symbol: "XLM",
    amount: 14500,
    valueUsd: 16530,
    custody: true,
    status: "active"
  },
  {
    id: "asset-2",
    name: "USDC Stellar",
    symbol: "USDC",
    amount: 8200,
    valueUsd: 8200,
    custody: true,
    status: "active"
  },
  {
    id: "asset-3",
    name: "Tokenized Treasury",
    symbol: "T-BILL",
    amount: 1200,
    valueUsd: 12180,
    custody: false,
    status: "pending"
  }
];

export const seedBeneficiaries: Beneficiary[] = [
  {
    id: "beneficiary-1",
    name: "Valeria Ruiz",
    address: "GBH7...YQ3Q",
    percentage: 50,
    status: "verified"
  },
  {
    id: "beneficiary-2",
    name: "Carlos Vega",
    address: "GA92...8ZQX",
    percentage: 35,
    status: "verified"
  },
  {
    id: "beneficiary-3",
    name: "Lina Mora",
    address: "GC4R...TUX1",
    percentage: 15,
    status: "pending"
  }
];

export const seedActivity: Activity[] = [
  {
    id: "activity-1",
    title: "Proof-of-life verified",
    detail: "Score 86.2% · Oracle sync confirmed",
    timestamp: "Today · 09:42",
    type: "verification"
  },
  {
    id: "activity-2",
    title: "Deposit received",
    detail: "+3,500 XLM added to custody",
    timestamp: "Today · 07:18",
    type: "deposit"
  },
  {
    id: "activity-3",
    title: "Escrow milestone prepared",
    detail: "Trustless Work escrow queued",
    timestamp: "Yesterday · 19:10",
    type: "distribution"
  },
  {
    id: "activity-4",
    title: "Risk signal detected",
    detail: "Liveness score below 40% threshold",
    timestamp: "Yesterday · 08:55",
    type: "alert"
  }
];
