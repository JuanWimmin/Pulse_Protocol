
  import { useEffect, useMemo, useState } from "react";
  import { checkFreighter, connectFreighter } from "./lib/freighter";
  import { authenticateWithWallet, graphQLRequest } from "./lib/api";

  type SectionKey =
    | "overview"
    | "dashboard"
    | "assets"
    | "beneficiaries"
    | "activity";

  type WalletState = {
    status: "idle" | "connected" | "missing" | "error" | "authenticating";
    address?: string;
    network?: string;
    error?: string;
  };

  type Vault = {
    id: string;
    owner: string;
    status: string;
    createdAt: string;
  };

  type Asset = {
    id: string;
    name: string;
    symbol: string;
    amount: number;
    valueUsd: number;
    custody: boolean;
    status: string;
    createdAt: string;
  };

  type Beneficiary = {
    address: string;
    percentage: number;
    claimed: boolean;
    claimedAt?: string | null;
  };

  type ActivityEvent = {
    id: string;
    title: string;
    detail?: string | null;
    kind: string;
    createdAt: string;
  };

  type LivenessData = {
    score: number;
    lastVerified: string;
    totalVerifications: number;
  };

  const sections: { key: SectionKey; label: string; description: string }[] = [
    {
      key: "overview",
      label: "Overview",
      description: "Start here: system overview and quick actions"
    },
    {
      key: "dashboard",
      label: "Control Panel",
      description: "Portfolio health, custody metrics, and signals"
    },
    {
      key: "assets",
      label: "Assets",
      description: "Track and manage assets held in custody"
    },
    {
      key: "beneficiaries",
      label: "Beneficiaries",
      description: "Maintain distribution rules and beneficiaries"
    },
    {
      key: "activity",
      label: "Activity",
      description: "Monitor lifecycle events and compliance actions"
    }
  ];

  const currency = new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: 0
  });

  const percent = new Intl.NumberFormat("en-US", {
    style: "percent",
    maximumFractionDigits: 0
  });

  const formatCompact = (value: number) =>
    new Intl.NumberFormat("en-US", {
      notation: "compact",
      maximumFractionDigits: 1
    }).format(value);

  const QUERIES = {
    dashboard: `
      query Dashboard($limit: Int) {
        myVaults { id owner status createdAt }
        myAssets { id name symbol amount valueUsd custody status createdAt }
        myLiveness { score lastVerified totalVerifications }
        activityFeed(limit: $limit) { id title detail kind createdAt }
      }
    `,
    beneficiaries: `
      query Beneficiaries($vaultId: UUID!) {
        beneficiaries(vaultId: $vaultId) { address percentage claimed claimedAt }
      }
    `
  };

  const MUTATIONS = {
    createVault: `
      mutation CreateVault($input: CreateVaultInput!) {
        createVault(input: $input) { id owner status createdAt }
      }
    `,
    addAsset: `
      mutation AddAsset($input: AddAssetInput!) {
        addAsset(input: $input) { id name symbol amount valueUsd custody status createdAt }
      }
    `,
    removeAsset: `
      mutation RemoveAsset($id: UUID!) {
        removeAsset(id: $id)
      }
    `,
    setBeneficiaries: `
      mutation SetBeneficiaries($vaultId: UUID!, $beneficiaries: [BeneficiaryInput!]!) {
        setBeneficiaries(vaultId: $vaultId, beneficiaries: $beneficiaries) {
          address percentage claimed claimedAt
        }
      }
    `,
    logActivity: `
      mutation LogActivity($input: ActivityInput!) {
        logActivity(input: $input) { id title detail kind createdAt }
      }
    `,
    removeActivity: `
      mutation RemoveActivity($id: UUID!) {
        removeActivity(id: $id)
      }
    `
  };

  export default function App() {
    const [activeSection, setActiveSection] = useState<SectionKey>("overview");
    const [wallet, setWallet] = useState<WalletState>({ status: "idle" });
    const [authToken, setAuthToken] = useState<string | null>(
      () => localStorage.getItem("pp_token")
    );
    const [vaults, setVaults] = useState<Vault[]>([]);
    const [assets, setAssets] = useState<Asset[]>([]);
    const [activity, setActivity] = useState<ActivityEvent[]>([]);
    const [beneficiaries, setBeneficiaries] = useState<Beneficiary[]>([]);
    const [liveness, setLiveness] = useState<LivenessData | null>(null);
    const [selectedVaultId, setSelectedVaultId] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isDark, setIsDark] = useState(true);

    const [assetForm, setAssetForm] = useState({
      name: "",
      symbol: "",
      amount: "",
      valueUsd: "",
      custody: true
    });

    const [beneficiaryForm, setBeneficiaryForm] = useState({
      name: "",
      address: "",
      percentage: ""
    });

    const [activityForm, setActivityForm] = useState({
      title: "",
      detail: "",
      kind: "verification"
    });

    const [vaultForm, setVaultForm] = useState({
      token: "XLM",
      initialDeposit: ""
    });

    const handleLogout = () => {
      localStorage.removeItem("pp_token");
      setAuthToken(null);
      setWallet({ status: "idle" });
      setVaults([]);
      setAssets([]);
      setActivity([]);
      setBeneficiaries([]);
      setLiveness(null);
      setSelectedVaultId(null);
    };

    useEffect(() => {
      document.body.classList.toggle("light-mode", !isDark);
    }, [isDark]);

    useEffect(() => {
      let mounted = true;
      checkFreighter().then((status) => {
        if (!mounted) return;
        if (!status.installed) {
          setWallet({ status: "missing" });
          return;
        }
        if (status.address) {
          setWallet({
            status: "connected",
            address: status.address,
            network: status.network
          });
        }
      });
      return () => {
        mounted = false;
      };
    }, []);

    useEffect(() => {
      if (!authToken) return;
      void loadDashboard();
    }, [authToken]);

    useEffect(() => {
      if (!authToken || !selectedVaultId) return;
      void loadBeneficiaries(selectedVaultId);
    }, [authToken, selectedVaultId]);

    const totals = useMemo(() => {
      const totalValue = assets.reduce((sum, asset) => sum + asset.valueUsd, 0);
      const custodyValue = assets
        .filter((asset) => asset.custody)
        .reduce((sum, asset) => sum + asset.valueUsd, 0);
      const custodyCount = assets.filter((asset) => asset.custody).length;
      const activeAssets = assets.filter((asset) => asset.status === "active")
        .length;
      return { totalValue, custodyValue, custodyCount, activeAssets };
    }, [assets]);

    const latestActivity = activity.slice(0, 4);

    const handleConnectWallet = async () => {
      setError(null);
      setWallet((prev) => ({ ...prev, status: "authenticating" }));
      const result = await connectFreighter();
      if (!result.installed) {
        setWallet({ status: "missing" });
        return;
      }
      if (result.error || !result.address) {
        setWallet({ status: "error", error: result.error });
        return;
      }

      try {
        const auth = await authenticateWithWallet(result.address);
        localStorage.setItem("pp_token", auth.token);
        setAuthToken(auth.token);
        setWallet({
          status: "connected",
          address: result.address,
          network: result.network
        });
      } catch (err) {
        setWallet({
          status: "error",
          error: err instanceof Error ? err.message : "Auth failed"
        });
      }
    };

    const loadDashboard = async () => {
      if (!authToken) return;
      setLoading(true);
      setError(null);
      try {
        const data = await graphQLRequest<{
          myVaults: Vault[];
          myAssets: Asset[];
          myLiveness: LivenessData | null;
          activityFeed: ActivityEvent[];
        }>(authToken, QUERIES.dashboard, { limit: 25 });

        setVaults(data.myVaults);
        setAssets(data.myAssets);
        setLiveness(data.myLiveness);
        setActivity(data.activityFeed);

        if (!selectedVaultId && data.myVaults.length > 0) {
          setSelectedVaultId(data.myVaults[0].id);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load data");
      } finally {
        setLoading(false);
      }
    };

    const loadBeneficiaries = async (vaultId: string) => {
      if (!authToken) return;
      try {
        const data = await graphQLRequest<{ beneficiaries: Beneficiary[] }>(
          authToken,
          QUERIES.beneficiaries,
          { vaultId }
        );
        setBeneficiaries(data.beneficiaries);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load beneficiaries");
      }
    };

    const handleCreateVault = async () => {
      if (!authToken) return;
      setError(null);
      try {
        const data = await graphQLRequest<{ createVault: Vault }>(
          authToken,
          MUTATIONS.createVault,
          {
            input: {
              token: vaultForm.token,
              initialDeposit: vaultForm.initialDeposit || null
            }
          }
        );
        setVaults((prev) => [data.createVault, ...prev]);
        if (!selectedVaultId) {
          setSelectedVaultId(data.createVault.id);
        }
        setVaultForm({ token: "XLM", initialDeposit: "" });
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to create vault");
      }
    };

    const handleAddAsset = async () => {
      if (!authToken || !assetForm.name || !assetForm.symbol) return;
      setError(null);
      const amount = Number(assetForm.amount || 0);
      const valueUsd = Number(assetForm.valueUsd || 0);
      try {
        const data = await graphQLRequest<{ addAsset: Asset }>(
          authToken,
          MUTATIONS.addAsset,
          {
            input: {
              name: assetForm.name,
              symbol: assetForm.symbol.toUpperCase(),
              amount,
              valueUsd,
              custody: assetForm.custody
            }
          }
        );
        setAssets((prev) => [data.addAsset, ...prev]);
        setAssetForm({
          name: "",
          symbol: "",
          amount: "",
          valueUsd: "",
          custody: true
        });
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to add asset");
      }
    };

    const handleRemoveAsset = async (id: string) => {
      if (!authToken) return;
      setError(null);
      try {
        await graphQLRequest<{ removeAsset: boolean }>(
          authToken,
          MUTATIONS.removeAsset,
          { id }
        );
        setAssets((prev) => prev.filter((asset) => asset.id !== id));
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to remove asset");
      }
    };

    const handleAddBeneficiary = () => {
      if (!beneficiaryForm.name || !beneficiaryForm.address) return;
      const percentage = Number(beneficiaryForm.percentage || 0);
      const newBeneficiary: Beneficiary = {
        address: beneficiaryForm.address,
        percentage: Math.round(percentage * 100),
        claimed: false
      };
      setBeneficiaries((prev) => [newBeneficiary, ...prev]);
      setBeneficiaryForm({ name: "", address: "", percentage: "" });
    };

    const handleRemoveBeneficiary = (address: string) => {
      setBeneficiaries((prev) =>
        prev.filter((item) => item.address !== address)
      );
    };

    const handleSaveBeneficiaries = async () => {
      if (!authToken || !selectedVaultId) return;
      setError(null);
      const total = beneficiaries.reduce((sum, b) => sum + b.percentage, 0);
      if (total !== 10000) {
        setError("Beneficiary allocations must sum to 100%");
        return;
      }
      try {
        const data = await graphQLRequest<{ setBeneficiaries: Beneficiary[] }>(
          authToken,
          MUTATIONS.setBeneficiaries,
          {
            vaultId: selectedVaultId,
            beneficiaries: beneficiaries.map((b) => ({
              address: b.address,
              percentage: b.percentage
            }))
          }
        );
        setBeneficiaries(data.setBeneficiaries);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to save beneficiaries");
      }
    };

    const handleAddActivity = async () => {
      if (!authToken || !activityForm.title) return;
      setError(null);
      try {
        const data = await graphQLRequest<{ logActivity: ActivityEvent }>(
          authToken,
          MUTATIONS.logActivity,
          {
            input: {
              title: activityForm.title,
              detail: activityForm.detail || null,
              kind: activityForm.kind
            }
          }
        );
        setActivity((prev) => [data.logActivity, ...prev]);
        setActivityForm({ title: "", detail: "", kind: "verification" });
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to log activity");
      }
    };

    const handleRemoveActivity = async (id: string) => {
      if (!authToken) return;
      setError(null);
      try {
        await graphQLRequest<{ removeActivity: boolean }>(
          authToken,
          MUTATIONS.removeActivity,
          { id }
        );
        setActivity((prev) => prev.filter((item) => item.id !== id));
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to remove activity");
      }
    };

    const beneficiaryTotal = beneficiaries.reduce((sum, b) => sum + b.percentage, 0);

    return (
      <div className="app">
        <button
          className="theme-toggle"
          onClick={() => setIsDark(!isDark)}
          title="Toggle theme"
        >
          {isDark ? "○" : "●"}
        </button>
        <aside className="sidebar">
          <div className="brand">
            <span className="brand-mark">♦</span>
            <div>
              <p className="brand-title">Pulse</p>
              <p className="brand-subtitle">Inheritance Operations</p>
            </div>
          </div>

          <nav className="nav">
            <p className="nav-group-label">Main</p>
            {sections.map((section) => (
  <button
    key={section.key}
    className={`nav-item ${activeSection === section.key ? "active" : ""}`}
    onClick={() => setActiveSection(section.key)}
    data-tooltip={section.description}
  >
    <span>{section.label}</span>
  </button>
))}
          </nav>

          <div className="sidebar-footer">
            <button className="logout-btn" onClick={handleLogout}>
              Disconnect wallet
            </button>
            <div className="status-pill">
              <span className="pulse" />
              Testnet ready
            </div>
            <p className="footnote">
              Trustless Work escrow integration queued for release.
            </p>
          </div>
        </aside>

        <main className="content">
          <header className="topbar">
            <div>
              <p className="eyebrow">Operational Console</p>
              <h1>{sections.find((s) => s.key === activeSection)?.label}</h1>
            </div>
            <div className="wallet">
              {wallet.status === "connected" ? (
                <div className="wallet-info">
                  <div>
                    <p className="wallet-label">Wallet connected</p>
                    <p className="wallet-address numeric">
                      {wallet.address?.slice(0, 6)}...{wallet.address?.slice(-4)}
                    </p>
                  </div>
                  <span className="wallet-network">
                    {wallet.network || "network"}
                  </span>
                </div>
              ) : (
                <div className="wallet-cta">
                  <button className="primary" onClick={handleConnectWallet}>
                    {wallet.status === "authenticating"
                      ? "Authenticating..."
                      : "Connect Freighter"}
                  </button>
                  {wallet.status === "missing" && (
                    <span className="wallet-hint">
                      Install the Freighter wallet extension
                    </span>
                  )}
                  {wallet.status === "error" && (
                    <span className="wallet-hint error">{wallet.error}</span>
                  )}
                </div>
              )}
            </div>
          </header>

          {error && (
            <div className="alert">
              <strong>Action needed:</strong> {error}
            </div>
          )}

          {activeSection === "overview" && (
            <section className="overview">
              <div className="hero card">
                <div>
                  <p className="eyebrow">Pulse Protocol MVP</p>
                  <h2>Operational headquarters for crypto inheritance</h2>
                  <p className="muted">
                    Track assets, manage beneficiaries, and monitor liveness
                    signals. Data is synced from your PostgreSQL-backed backend in
                    real time.
                  </p>
                  <div className="hero-actions">
                    <button
                      className="primary"
                      onClick={() => setActiveSection("dashboard")}
                    >
                      Open control panel
                    </button>
                    <button
                      className="ghost"
                      onClick={() => setActiveSection("assets")}
                    >
                      Manage assets
                    </button>
                    <button
                      className="ghost"
                      onClick={() => setActiveSection("beneficiaries")}
                    >
                      Manage beneficiaries
                    </button>
                  </div>
                </div>
                <div className="hero-metrics">
                  <div>
                    <p className="metric numeric">
                      {currency.format(totals.totalValue)}
                    </p>
                    <p className="metric-label">Total assets under custody</p>
                  </div>
                  <div className="hero-grid">
                    <div>
                      <p className="metric-value numeric">
                        {totals.activeAssets}
                      </p>
                      <p className="metric-label">Active assets</p>
                    </div>
                    <div>
                      <p className="metric-value numeric">{vaults.length}</p>
                      <p className="metric-label">Vaults created</p>
                    </div>
                    <div>
                      <p className="metric-value numeric">
                        {liveness ? `${liveness.score / 100}%` : "--"}
                      </p>
                      <p className="metric-label">Liveness score</p>
                    </div>
                  </div>
                </div>
              </div>

              <div className="overview-grid">
                <div className="card">
                  <div className="card-header">
                    <h3>Quick actions</h3>
                    <span className="badge subtle">Shortcuts</span>
                  </div>
                  <div className="quick-actions">
                    <button
                      className="ghost"
                      onClick={() => setActiveSection("activity")}
                    >
                      Log activity event
                    </button>
                    <button
                      className="ghost"
                      onClick={() => setActiveSection("assets")}
                    >
                      Add asset
                    </button>
                    <button
                      className="ghost"
                      onClick={() => setActiveSection("beneficiaries")}
                    >
                      Set beneficiaries
                    </button>
                  </div>
                </div>

                <div className="card">
                  <div className="card-header">
                    <h3>Vaults</h3>
                    <span className="badge">{formatCompact(vaults.length)}</span>
                  </div>
                  {vaults.length === 0 ? (
                    <p className="muted">Create your first vault to begin.</p>
                  ) : (
                    <ul className="stack">
                      {vaults.slice(0, 4).map((vault) => (
                        <li key={vault.id}>
                          <div>
                            <p className="list-title">Vault {vault.id.slice(0, 6)}</p>
                            <p className="list-subtitle">Status - {vault.status}</p>
                          </div>
                          <span className="list-meta">
                            {new Date(vault.createdAt).toLocaleDateString()}
                          </span>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>

                <div className="card form-card">
                  <div className="card-header">
                    <h3>Create vault</h3>
                    <span className="badge subtle">Backend powered</span>
                  </div>
                  <div className="form">
                    <label>
                      Token
                      <input
                        value={vaultForm.token}
                        onChange={(event) =>
                          setVaultForm((prev) => ({
                            ...prev,
                            token: event.target.value
                          }))
                        }
                        placeholder="XLM"
                      />
                    </label>
                    <label>
                      Initial deposit (optional)
                      <input
                        value={vaultForm.initialDeposit}
                        onChange={(event) =>
                          setVaultForm((prev) => ({
                            ...prev,
                            initialDeposit: event.target.value
                          }))
                        }
                        placeholder="1000"
                        type="number"
                      />
                    </label>
                    <button className="primary" onClick={handleCreateVault}>
                      Create vault
                    </button>
                  </div>
                </div>
              </div>
            </section>
          )}

          {activeSection === "dashboard" && (
            <section className="grid dashboard">
              <div className="card jumbo">
                <div className="card-header">
                  <h2>Total Asset Value</h2>
                  <span className="badge">
                    {loading ? "Syncing..." : "Live from database"}
                  </span>
                </div>
                <p className="metric numeric">
                  {currency.format(totals.totalValue)}
                </p>
                <div className="metric-row">
                  <div>
                    <p className="metric-label">Assets in custody</p>
                    <p className="metric-value numeric">
                      {currency.format(totals.custodyValue)}
                    </p>
                  </div>
                  <div>
                    <p className="metric-label">Active assets</p>
                    <p className="metric-value numeric">
                      {totals.activeAssets}
                    </p>
                  </div>
                  <div>
                    <p className="metric-label">Custody ratio</p>
                    <p className="metric-value numeric">
                      {percent.format(
                        totals.totalValue === 0
                          ? 0
                          : totals.custodyValue / totals.totalValue
                      )}
                    </p>
                  </div>
                </div>
              </div>

              <div className="card">
                <div className="card-header">
                  <h3>Liveness pulse</h3>
                  <span className="badge subtle">Proof-of-life</span>
                </div>
                {liveness ? (
                  <div className="stack">
                    <div>
                      <p className="metric-value numeric">
                        {liveness.score / 100}%
                      </p>
                      <p className="metric-label">
                        Last verified {new Date(liveness.lastVerified).toLocaleString()}
                      </p>
                    </div>
                    <div>
                      <p className="metric-label">Total verifications</p>
                      <p className="metric-value numeric">
                        {liveness.totalVerifications}
                      </p>
                    </div>
                  </div>
                ) : (
                  <p className="muted">No verifications yet.</p>
                )}
              </div>

              <div className="card">
                <div className="card-header">
                  <h3>Recent Activity</h3>
                  <span className="badge subtle">{activity.length} events</span>
                </div>
                <ul className="activity-list">
                  {latestActivity.length === 0 ? (
                    <li className="empty">No activity logged yet.</li>
                  ) : (
                    latestActivity.map((item) => (
                      <li key={item.id} className={`activity ${item.kind}`}>
                        <div>
                          <p className="list-title">{item.title}</p>
                          <p className="list-subtitle">{item.detail || "--"}</p>
                        </div>
                        <span className="list-meta">
                          {new Date(item.createdAt).toLocaleString()}
                        </span>
                      </li>
                    ))
                  )}
                </ul>
              </div>

              <div className="card">
                <div className="card-header">
                  <h3>Beneficiary Coverage</h3>
                  <span className="badge subtle">
                    {formatCompact(beneficiaries.length)} entries
                  </span>
                </div>
                <ul className="stack">
                  {beneficiaries.length === 0 ? (
                    <li className="empty">No beneficiaries set yet.</li>
                  ) : (
                    beneficiaries.map((b) => (
                      <li key={b.address}>
                        <div>
                          
<p className="list-title">
  {b.address.slice(0, 6)}...{b.address.slice(-4)}
</p>
                          <p className="list-subtitle">
                              Allocation {b.percentage / 100}%
                          </p>
                        </div>
                        <span className={`status ${b.claimed ? "verified" : "pending"}`}>
                          {b.claimed ? "claimed" : "pending"}
                        </span>
                      </li>
                    ))
                  )}
                </ul>
              </div>
            </section>
          )}

          {activeSection === "assets" && (
            <section className="grid wide">
              <div className="card">
                <div className="card-header">
                  <h3>Asset Inventory</h3>
                  <span className="badge">
                    {formatCompact(assets.length)} tracked assets
                  </span>
                </div>
                <div className="table">
                  <div className="table-row table-header">
                    <span>Asset</span>
                    <span>Holdings</span>
                    <span>Value (USD)</span>
                    <span>Status</span>
                    <span>Action</span>
                  </div>
                  {assets.length === 0 ? (
                    <div className="table-row empty-row">No assets yet.</div>
                  ) : (
                    assets.map((asset) => (
                      <div className="table-row" key={asset.id}>
                        <span>
                          <strong>{asset.symbol}</strong>
                          <small>{asset.name}</small>
                        </span>
                        <span>
                          <span className="numeric">
                            {asset.amount.toLocaleString()}
                          </span>{" "}
                          {asset.symbol}
                        </span>
                        <span className="numeric">
                          {currency.format(asset.valueUsd)}
                        </span>
                        <span className={`status ${asset.status}`}>
                          {asset.custody ? "Custody" : "External"} - {asset.status}
                        </span>
                        <button
                          className="ghost"
                          onClick={() => handleRemoveAsset(asset.id)}
                        >
                          Remove
                        </button>
                      </div>
                    ))
                  )}
                </div>
              </div>

              <div className="card form-card">
                <div className="card-header">
                  <h3>Add Asset</h3>
                  <span className="badge subtle">Database entry</span>
                </div>
                <div className="form">
                  <label>
                    Asset name
                    <input
                      value={assetForm.name}
                      onChange={(event) =>
                        setAssetForm((prev) => ({
                          ...prev,
                          name: event.target.value
                        }))
                      }
                      placeholder="Stellar Lumens"
                    />
                  </label>
                  <label>
                    Symbol
                    <input
                      value={assetForm.symbol}
                      onChange={(event) =>
                        setAssetForm((prev) => ({
                          ...prev,
                          symbol: event.target.value
                        }))
                      }
                      placeholder="XLM"
                    />
                  </label>
                  <label>
                    Amount
                    <input
                      value={assetForm.amount}
                      onChange={(event) =>
                        setAssetForm((prev) => ({
                          ...prev,
                          amount: event.target.value
                        }))
                      }
                      placeholder="14500"
                      type="number"
                    />
                  </label>
                  <label>
                    USD value
                    <input
                      value={assetForm.valueUsd}
                      onChange={(event) =>
                        setAssetForm((prev) => ({
                          ...prev,
                          valueUsd: event.target.value
                        }))
                      }
                      placeholder="16530"
                      type="number"
                    />
                  </label>
                  <label className="toggle">
                    In custody
                    <input
                      type="checkbox"
                      checked={assetForm.custody}
                      onChange={(event) =>
                        setAssetForm((prev) => ({
                          ...prev,
                          custody: event.target.checked
                        }))
                      }
                    />
                  </label>
                  <button className="primary" onClick={handleAddAsset}>
                    Add asset
                  </button>
                </div>
              </div>
            </section>
          )}

          {activeSection === "beneficiaries" && (
            <section className="grid wide">
              <div className="card">
                <div className="card-header">
                  <h3>Beneficiary Matrix</h3>
                  <span className="badge">
                    {formatCompact(beneficiaries.length)} contacts
                  </span>
                </div>
                <div className="selector-row">
                  <label>
                    Vault
                    <select
                      value={selectedVaultId ?? ""}
                      onChange={(event) =>
                        setSelectedVaultId(event.target.value || null)
                      }
                    >
                      <option value="">Select a vault</option>
                      {vaults.map((vault) => (
                        <option key={vault.id} value={vault.id}>
                          Vault {vault.id.slice(0, 6)} - {vault.status}
                        </option>
                      ))}
                    </select>
                  </label>
                  <div className="allocation">
                    <p className="metric-value numeric">
                      {beneficiaryTotal / 100}%
                    </p>
                    <p className="metric-label">Total allocation</p>
                  </div>
                </div>
                <div className="table">
                  <div className="table-row table-header">
                    <span>Beneficiary</span>
                    <span>Address</span>
                    <span>Allocation</span>
                    <span>Status</span>
                    <span>Action</span>
                  </div>
                  {beneficiaries.length === 0 ? (
                    <div className="table-row empty-row">
                      No beneficiaries assigned yet.
                    </div>
                  ) : (
                    beneficiaries.map((person) => (
                      <div className="table-row" key={person.address}>
                        <span>
                          <strong>{person.address.slice(0, 6)}</strong>
                          <small>Beneficiary</small>
                        </span>
                        
<span className="address-cell" title={person.address}>
  {person.address.slice(0, 6)}...{person.address.slice(-4)}
</span>
                        <span className="numeric">
                          {person.percentage / 100}%
                        </span>
                        <span
                          className={`status ${person.claimed ? "verified" : "pending"}`}
                        >
                          {person.claimed ? "claimed" : "pending"}
                        </span>
                        <button
                          className="ghost"
                          onClick={() => handleRemoveBeneficiary(person.address)}
                        >
                          Remove
                        </button>
                      </div>
                    ))
                  )}
                </div>
                <div className="actions-row">
                  <button className="primary" onClick={handleSaveBeneficiaries}>
                    Save allocations
                  </button>
                  <span className="hint">
                    Total must be exactly 100% to save.
                  </span>
                </div>
              </div>

              <div className="card form-card">
                <div className="card-header">
                  <h3>Add Beneficiary</h3>
                  <span className="badge subtle">Draft list</span>
                </div>
                <div className="form">
                  <label>
                    Full name
                    <input
                      value={beneficiaryForm.name}
                      onChange={(event) =>
                        setBeneficiaryForm((prev) => ({
                          ...prev,
                          name: event.target.value
                        }))
                      }
                      placeholder="Valeria Ruiz"
                    />
                  </label>
                  <label>
                    Stellar address
                    <input
                      value={beneficiaryForm.address}
                      onChange={(event) =>
                        setBeneficiaryForm((prev) => ({
                          ...prev,
                          address: event.target.value
                        }))
                      }
                      placeholder="GBH7..."
                    />
                  </label>
                  <label>
                    Allocation %
                    <input
                      value={beneficiaryForm.percentage}
                      onChange={(event) =>
                        setBeneficiaryForm((prev) => ({
                          ...prev,
                          percentage: event.target.value
                        }))
                      }
                      placeholder="25"
                      type="number"
                    />
                  </label>
                  <button className="primary" onClick={handleAddBeneficiary}>
                    Add beneficiary
                  </button>
                </div>
              </div>
            </section>
          )}

          {activeSection === "activity" && (
            <section className="grid wide">
              <div className="card">
                <div className="card-header">
                  <h3>Activity Stream</h3>
                  <span className="badge">Last updates</span>
                </div>
                <div className="activity-stream">
                  {activity.length === 0 ? (
                    <div className="empty">No activity logged yet.</div>
                  ) : (
                    activity.map((item) => (
                      <div className={`activity-row ${item.kind}`} key={item.id}>
                        <div>
                          <p className="list-title">{item.title}</p>
                          <p className="list-subtitle">{item.detail || "--"}</p>
                        </div>
                        <span className="list-meta">
                          {new Date(item.createdAt).toLocaleString()}
                        </span>
                        <button
                          className="ghost"
                          onClick={() => handleRemoveActivity(item.id)}
                        >
                          Remove
                        </button>
                      </div>
                    ))
                  )}
                </div>
              </div>

              <div className="card form-card">
                <div className="card-header">
                  <h3>Log New Activity</h3>
                  <span className="badge subtle">Manual record</span>
                </div>
                <div className="form">
                  <label>
                    Title
                    <input
                      value={activityForm.title}
                      onChange={(event) =>
                        setActivityForm((prev) => ({
                          ...prev,
                          title: event.target.value
                        }))
                      }
                      placeholder="Verification submitted"
                    />
                  </label>
                  <label>
                    Detail
                    <input
                      value={activityForm.detail}
                      onChange={(event) =>
                        setActivityForm((prev) => ({
                          ...prev,
                          detail: event.target.value
                        }))
                      }
                      placeholder="Score 82% - On-chain confirmation pending"
                    />
                  </label>
                  <label>
                    Type
                    <select
                      value={activityForm.kind}
                      onChange={(event) =>
                        setActivityForm((prev) => ({
                          ...prev,
                          kind: event.target.value
                        }))
                      }
                    >
                      <option value="verification">Verification</option>
                      <option value="deposit">Deposit</option>
                      <option value="distribution">Distribution</option>
                      <option value="alert">Alert</option>
                    </select>
                  </label>
                  <button className="primary" onClick={handleAddActivity}>
                    Add activity
                  </button>
                </div>
              </div>
            </section>
          )}
        </main>
      </div>
    );
  }
