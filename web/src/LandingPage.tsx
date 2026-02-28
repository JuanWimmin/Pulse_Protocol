import { useState } from "react";
import { checkFreighter, connectFreighter } from "./lib/freighter";
import { authenticateWithWallet } from "./lib/api";

type WalletState = {
  status: "idle" | "connected" | "missing" | "error" | "authenticating";
  address?: string;
  network?: string;
  error?: string;
};

type Props = {
  onEnterApp: () => void;
};

export default function LandingPage({ onEnterApp }: Props) {
  const [isDark, setIsDark] = useState(true);
  const [wallet, setWallet] = useState<WalletState>({ status: "idle" });

  const toggleTheme = () => setIsDark((prev) => !prev);

  const handleConnectWallet = async () => {
    setWallet({ status: "authenticating" });

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
      setWallet({
        status: "connected",
        address: result.address,
        network: result.network,
      });
    } catch (err) {
      setWallet({
        status: "error",
        error: err instanceof Error ? err.message : "Auth failed",
      });
    }
  };

  const handleDisconnect = () => {
    localStorage.removeItem("pp_token");
    setWallet({ status: "idle" });
  };

  const walletConnected = wallet.status === "connected";
  const isLoading = wallet.status === "authenticating";

  return (
    <div data-theme={isDark ? "dark" : "light"} style={{ colorScheme: isDark ? "dark" : "light" }}>
      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=DM+Sans:ital,wght@0,300;0,400;0,500;0,600;0,700;1,300&family=Syne:wght@700;800&display=swap');

        [data-theme="dark"] {
          --bg:            #0d0b1e;
          --bg2:           #150d2e;
          --bg3:           #0f0f1f;
          --surface:       rgba(255,255,255,0.04);
          --surface-hover: rgba(255,255,255,0.07);
          --border:        rgba(255,255,255,0.08);
          --border-accent: rgba(162,92,246,0.3);
          --text:          #e8e6ff;
          --text-muted:    #7b78a0;
          --primary:       #a25cf6;
          --primary2:      #6366f1;
          --success:       #4ade80;
          --shadow-card:   0 8px 32px rgba(0,0,0,0.4);
          --shadow-glow:   0 0 40px rgba(162,92,246,0.15);
        }
        [data-theme="light"] {
          --bg:            #f4f2ff;
          --bg2:           #ede9ff;
          --bg3:           #f9f8ff;
          --surface:       rgba(255,255,255,0.85);
          --surface-hover: rgba(255,255,255,0.95);
          --border:        rgba(162,92,246,0.15);
          --border-accent: rgba(162,92,246,0.35);
          --text:          #1a1040;
          --text-muted:    #7065a0;
          --primary:       #7c3aed;
          --primary2:      #4f46e5;
          --success:       #16a34a;
          --shadow-card:   0 8px 32px rgba(100,60,200,0.12);
          --shadow-glow:   0 0 40px rgba(124,58,237,0.08);
        }

        .lp-root {
          font-family: 'DM Sans', sans-serif;
          background: var(--bg);
          color: var(--text);
          min-height: 100vh;
          transition: background 0.35s ease, color 0.35s ease;
          overflow-x: hidden;
        }

        /* ATMOSPHERE */
        .lp-atmosphere {
          position: fixed;
          inset: 0;
          pointer-events: none;
          z-index: 0;
          overflow: hidden;
        }
        .lp-orb {
          position: absolute;
          border-radius: 50%;
        }
        .lp-orb-1 {
          width: 600px; height: 600px;
          top: -200px; right: -150px;
          background: radial-gradient(circle, rgba(162,92,246,0.18) 0%, transparent 65%);
        }
        [data-theme="light"] .lp-orb-1 {
          background: radial-gradient(circle, rgba(124,58,237,0.1) 0%, transparent 65%);
        }
        .lp-orb-2 {
          width: 400px; height: 400px;
          bottom: 100px; left: -100px;
          background: radial-gradient(circle, rgba(99,102,241,0.12) 0%, transparent 65%);
        }
        [data-theme="light"] .lp-orb-2 {
          background: radial-gradient(circle, rgba(79,70,229,0.08) 0%, transparent 65%);
        }
        .lp-orb-3 {
          width: 300px; height: 300px;
          top: 50%; left: 50%;
          transform: translate(-50%, -50%);
          background: radial-gradient(circle, rgba(162,92,246,0.05) 0%, transparent 70%);
        }

        /* NAV */
        .lp-nav {
          position: fixed;
          top: 0; left: 0; right: 0;
          z-index: 100;
          padding: 0 48px;
          height: 68px;
          display: flex;
          align-items: center;
          justify-content: space-between;
          background: rgba(13,11,30,0.7);
          backdrop-filter: blur(20px);
          border-bottom: 1px solid var(--border);
          transition: background 0.35s ease, border-color 0.35s ease;
        }
        [data-theme="light"] .lp-nav {
          background: rgba(244,242,255,0.8);
        }
        .lp-nav-logo {
          font-family: 'Syne', sans-serif;
          font-size: 1.25rem;
          font-weight: 800;
          color: var(--text);
          letter-spacing: -0.5px;
          display: flex;
          align-items: center;
          gap: 6px;
          text-decoration: none;
          background: none;
          border: none;
          cursor: pointer;
        }
        .lp-diamond { color: var(--primary); font-size: 0.9rem; }
        .lp-nav-links {
          display: flex;
          align-items: center;
          gap: 32px;
        }
        .lp-nav-links a {
          font-size: 0.85rem;
          font-weight: 500;
          color: var(--text-muted);
          text-decoration: none;
          transition: color 0.2s;
        }
        .lp-nav-links a:hover { color: var(--text); }
        .lp-nav-actions {
          display: flex;
          align-items: center;
          gap: 10px;
        }
        .lp-theme-toggle {
          width: 36px; height: 36px;
          border-radius: 999px;
          background: var(--surface);
          border: 1px solid var(--border);
          color: var(--text-muted);
          cursor: pointer;
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 0.8rem;
          transition: all 0.2s;
        }
        .lp-theme-toggle:hover { border-color: var(--border-accent); color: var(--primary); }
        .lp-btn-ghost {
          padding: 8px 18px;
          border-radius: 8px;
          border: 1px solid var(--border);
          background: transparent;
          color: var(--text-muted);
          font-family: 'DM Sans', sans-serif;
          font-size: 0.82rem;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s;
        }
        .lp-btn-ghost:hover { border-color: var(--border-accent); color: var(--text); }
        .lp-btn-primary {
          padding: 8px 20px;
          border-radius: 8px;
          border: none;
          background: linear-gradient(135deg, var(--primary), var(--primary2));
          color: #fff;
          font-family: 'DM Sans', sans-serif;
          font-size: 0.82rem;
          font-weight: 700;
          cursor: pointer;
          transition: opacity 0.2s, transform 0.2s;
          box-shadow: 0 4px 16px rgba(162,92,246,0.3);
        }
        .lp-btn-primary:hover { opacity: 0.9; transform: translateY(-1px); }
        .lp-btn-connected {
          padding: 8px 20px;
          border-radius: 8px;
          border: 1px solid rgba(74,222,128,0.3);
          background: rgba(74,222,128,0.15);
          color: #4ade80;
          font-family: 'DM Sans', sans-serif;
          font-size: 0.82rem;
          font-weight: 700;
          cursor: pointer;
          transition: all 0.2s;
        }

        /* HERO */
        .lp-hero {
          position: relative;
          z-index: 1;
          min-height: 100vh;
          display: grid;
          grid-template-columns: 1fr 1fr;
          align-items: center;
          gap: 60px;
          padding: 120px 80px 80px;
          max-width: 1280px;
          margin: 0 auto;
        }
        .lp-hero-left {
          animation: lp-fadeUp 0.7s ease both;
        }
        .lp-hero-right {
          animation: lp-fadeUp 0.7s ease 0.15s both;
        }
        @keyframes lp-fadeUp {
          from { opacity: 0; transform: translateY(24px); }
          to   { opacity: 1; transform: translateY(0); }
        }
        .lp-hero-tag {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          background: rgba(162,92,246,0.1);
          border: 1px solid rgba(162,92,246,0.25);
          padding: 5px 14px;
          border-radius: 999px;
          font-size: 0.72rem;
          font-weight: 600;
          color: var(--primary);
          margin-bottom: 24px;
          letter-spacing: 0.3px;
        }
        .lp-dot {
          width: 6px; height: 6px;
          border-radius: 50%;
          background: var(--primary);
          animation: lp-blink 2s ease-in-out infinite;
        }
        @keyframes lp-blink {
          0%,100% { opacity: 0.4; }
          50% { opacity: 1; }
        }
        .lp-hero-title {
          font-family: 'Syne', sans-serif;
          font-size: clamp(2.4rem, 4vw, 3.2rem);
          font-weight: 800;
          line-height: 1.1;
          letter-spacing: -1.5px;
          color: var(--text);
          margin-bottom: 20px;
        }
        .lp-hl {
          background: linear-gradient(90deg, var(--primary), var(--primary2));
          -webkit-background-clip: text;
          -webkit-text-fill-color: transparent;
        }
        .lp-hero-sub {
          font-size: 1rem;
          color: var(--text-muted);
          line-height: 1.7;
          margin-bottom: 36px;
          max-width: 420px;
          font-weight: 300;
        }
        .lp-hero-btns {
          display: flex;
          gap: 12px;
          flex-wrap: wrap;
        }
        .lp-hero-btns .lp-btn-primary { padding: 12px 28px; font-size: 0.9rem; }
        .lp-hero-btns .lp-btn-ghost   { padding: 12px 24px; font-size: 0.9rem; }
        .lp-hero-stats {
          display: flex;
          gap: 32px;
          margin-top: 48px;
          padding-top: 32px;
          border-top: 1px solid var(--border);
        }
        .lp-stat-val {
          font-family: 'Syne', sans-serif;
          font-size: 1.5rem;
          font-weight: 800;
          color: var(--text);
        }
        .lp-stat-lbl {
          font-size: 0.72rem;
          color: var(--text-muted);
          margin-top: 2px;
          font-weight: 500;
        }

        /* WALLET CARD */
        .lp-wallet-card {
          background: var(--surface);
          border: 1px solid var(--border);
          backdrop-filter: blur(24px);
          border-radius: 20px;
          padding: 28px;
          box-shadow: var(--shadow-card), var(--shadow-glow);
          position: relative;
          overflow: hidden;
          transition: background 0.35s ease, border-color 0.35s ease;
        }
        .lp-wallet-card::before {
          content: '';
          position: absolute;
          top: 0; left: 0; right: 0;
          height: 1px;
          background: linear-gradient(90deg, transparent, var(--primary), transparent);
          opacity: 0.6;
        }
        .lp-w-icon {
          width: 52px; height: 52px;
          border-radius: 14px;
          background: rgba(162,92,246,0.12);
          border: 1px solid rgba(162,92,246,0.2);
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 1.3rem;
          margin-bottom: 18px;
        }
        .lp-wc-title {
          font-family: 'Syne', sans-serif;
          font-size: 1.2rem;
          font-weight: 700;
          color: var(--text);
          margin-bottom: 8px;
        }
        .lp-wc-desc {
          font-size: 0.8rem;
          color: var(--text-muted);
          line-height: 1.6;
          margin-bottom: 22px;
        }
        .lp-wc-btn {
          width: 100%;
          padding: 13px;
          font-size: 0.88rem;
          border-radius: 8px;
          border: none;
          background: linear-gradient(135deg, var(--primary), var(--primary2));
          color: #fff;
          font-family: 'DM Sans', sans-serif;
          font-weight: 700;
          cursor: pointer;
          transition: opacity 0.2s, transform 0.2s;
          box-shadow: 0 4px 16px rgba(162,92,246,0.3);
          margin-bottom: 12px;
        }
        .lp-wc-btn:hover { opacity: 0.9; transform: translateY(-1px); }
        .lp-wallet-options {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 8px;
        }
        .lp-wallet-opt {
          padding: 10px 14px;
          border-radius: 8px;
          background: var(--surface);
          border: 1px solid var(--border);
          color: var(--text-muted);
          font-size: 0.75rem;
          font-weight: 600;
          cursor: pointer;
          text-align: center;
          transition: all 0.2s;
          font-family: 'DM Sans', sans-serif;
        }
        .lp-wallet-opt:hover { border-color: var(--border-accent); color: var(--primary); }
        .lp-wopt-icon { display: block; font-size: 1rem; margin-bottom: 4px; }

        /* Connected state */
        .lp-c-top {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 22px;
        }
        .lp-c-badge {
          display: flex;
          align-items: center;
          gap: 6px;
          background: rgba(74,222,128,0.1);
          border: 1px solid rgba(74,222,128,0.2);
          padding: 4px 10px;
          border-radius: 999px;
          font-size: 0.7rem;
          font-weight: 600;
          color: var(--success);
        }
        .lp-c-dot {
          width: 6px; height: 6px;
          border-radius: 50%;
          background: var(--success);
        }
        .lp-c-addr {
          font-size: 0.72rem;
          color: var(--text-muted);
          font-family: monospace;
          background: var(--surface);
          border: 1px solid var(--border);
          padding: 4px 10px;
          border-radius: 999px;
        }
        .lp-c-balance-label {
          font-size: 0.72rem;
          color: var(--text-muted);
          font-weight: 500;
          margin-bottom: 4px;
        }
        .lp-c-balance {
          font-family: 'Syne', sans-serif;
          font-size: 2.2rem;
          font-weight: 800;
          color: var(--text);
          margin-bottom: 4px;
          letter-spacing: -1px;
        }
        .lp-c-change {
          font-size: 0.75rem;
          font-weight: 600;
          color: var(--success);
          margin-bottom: 22px;
        }
        .lp-c-metrics {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 10px;
          margin-bottom: 20px;
        }
        .lp-c-metric {
          background: rgba(255,255,255,0.03);
          border: 1px solid var(--border);
          border-radius: 8px;
          padding: 12px 14px;
        }
        [data-theme="light"] .lp-c-metric { background: rgba(124,58,237,0.04); }
        .lp-c-metric-val { font-size: 1rem; font-weight: 700; color: var(--text); }
        .lp-c-metric-lbl { font-size: 0.62rem; color: var(--text-muted); margin-top: 2px; font-weight: 500; }
        .lp-c-metric-chg { font-size: 0.65rem; color: var(--success); font-weight: 600; margin-top: 1px; }

        /* Go to Dashboard button */
        .lp-btn-dashboard {
          width: 100%;
          padding: 12px;
          border-radius: 8px;
          border: none;
          background: linear-gradient(135deg, var(--primary), var(--primary2));
          color: white;
          font-family: 'DM Sans', sans-serif;
          font-size: 0.85rem;
          font-weight: 700;
          cursor: pointer;
          transition: all 0.2s;
          box-shadow: 0 4px 16px rgba(162,92,246,0.3);
          margin-bottom: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 8px;
        }
        .lp-btn-dashboard:hover { opacity: 0.9; transform: translateY(-1px); }
        .lp-btn-disconnect {
          width: 100%;
          padding: 11px;
          border-radius: 8px;
          border: 1px solid rgba(239,68,68,0.2);
          background: rgba(239,68,68,0.05);
          color: #f87171;
          font-family: 'DM Sans', sans-serif;
          font-size: 0.8rem;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s;
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 8px;
        }
        .lp-btn-disconnect:hover {
          background: rgba(239,68,68,0.12);
          border-color: rgba(239,68,68,0.35);
        }

        /* PARTNERS */
        .lp-partners-strip {
          position: relative;
          z-index: 1;
          border-top: 1px solid var(--border);
          border-bottom: 1px solid var(--border);
          padding: 28px 80px;
          display: flex;
          align-items: center;
          gap: 48px;
          justify-content: center;
        }
        .lp-partners-label {
          font-size: 0.68rem;
          color: var(--text-muted);
          text-transform: uppercase;
          letter-spacing: 1.5px;
          font-weight: 600;
          white-space: nowrap;
        }
        .lp-partner {
          font-size: 0.85rem;
          font-weight: 700;
          color: var(--text-muted);
          opacity: 0.5;
          letter-spacing: 0.5px;
          transition: opacity 0.2s;
          cursor: default;
        }
        .lp-partner:hover { opacity: 0.9; }

        /* SECTION */
        .lp-section {
          position: relative;
          z-index: 1;
          padding: 100px 80px;
          max-width: 1280px;
          margin: 0 auto;
        }
        .lp-section-pill {
          display: inline-flex;
          background: rgba(162,92,246,0.1);
          border: 1px solid rgba(162,92,246,0.2);
          color: var(--primary);
          padding: 4px 14px;
          border-radius: 999px;
          font-size: 0.7rem;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 1px;
          margin-bottom: 20px;
        }
        .lp-section-title {
          font-family: 'Syne', sans-serif;
          font-size: clamp(1.8rem, 3vw, 2.4rem);
          font-weight: 800;
          letter-spacing: -1px;
          color: var(--text);
          margin-bottom: 12px;
          max-width: 560px;
          line-height: 1.15;
        }
        .lp-section-sub {
          font-size: 0.9rem;
          color: var(--text-muted);
          max-width: 480px;
          line-height: 1.7;
          margin-bottom: 52px;
          font-weight: 300;
        }

        /* FEATURES */
        .lp-features-grid {
          display: grid;
          grid-template-columns: repeat(3, 1fr);
          gap: 20px;
        }
        .lp-feature-card {
          background: var(--surface);
          border: 1px solid var(--border);
          border-radius: 16px;
          padding: 28px;
          transition: all 0.25s ease;
          backdrop-filter: blur(10px);
        }
        .lp-feature-card:hover {
          background: var(--surface-hover);
          border-color: var(--border-accent);
          transform: translateY(-4px);
          box-shadow: var(--shadow-card);
        }
        .lp-feature-icon {
          width: 44px; height: 44px;
          border-radius: 12px;
          background: rgba(162,92,246,0.12);
          border: 1px solid rgba(162,92,246,0.2);
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 1rem;
          margin-bottom: 18px;
          color: var(--primary);
        }
        .lp-feature-title {
          font-size: 0.95rem;
          font-weight: 700;
          color: var(--text);
          margin-bottom: 8px;
        }
        .lp-feature-desc {
          font-size: 0.8rem;
          color: var(--text-muted);
          line-height: 1.65;
          font-weight: 300;
        }

        /* PRICING */
        .lp-pricing-grid {
          display: grid;
          grid-template-columns: repeat(3, 1fr);
          gap: 20px;
        }
        .lp-price-card {
          background: var(--surface);
          border: 1px solid var(--border);
          border-radius: 18px;
          padding: 32px 28px;
          transition: all 0.25s ease;
          backdrop-filter: blur(10px);
          position: relative;
        }
        .lp-price-card:hover {
          transform: translateY(-4px);
          box-shadow: var(--shadow-card);
        }
        .lp-price-card.featured {
          background: linear-gradient(135deg, rgba(162,92,246,0.2), rgba(99,102,241,0.15));
          border-color: rgba(162,92,246,0.4);
          box-shadow: 0 8px 32px rgba(162,92,246,0.2);
        }
        .lp-price-card.featured::before {
          content: '♦ Most Popular';
          position: absolute;
          top: -12px;
          left: 50%;
          transform: translateX(-50%);
          background: linear-gradient(135deg, var(--primary), var(--primary2));
          color: white;
          font-size: 0.68rem;
          font-weight: 700;
          padding: 4px 14px;
          border-radius: 999px;
          white-space: nowrap;
          letter-spacing: 0.3px;
        }
        .lp-plan-name {
          font-size: 0.75rem;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 1.5px;
          color: var(--text-muted);
          margin-bottom: 16px;
        }
        .lp-plan-price {
          font-family: 'Syne', sans-serif;
          font-size: 2.4rem;
          font-weight: 800;
          color: var(--text);
          letter-spacing: -1.5px;
          margin-bottom: 4px;
        }
        .lp-plan-price sup { font-size: 1rem; vertical-align: top; margin-top: 10px; display: inline-block; }
        .lp-plan-price span { font-size: 0.85rem; font-weight: 400; color: var(--text-muted); letter-spacing: 0; }
        .lp-plan-desc { font-size: 0.78rem; color: var(--text-muted); margin-bottom: 24px; line-height: 1.5; }
        .lp-plan-divider { height: 1px; background: var(--border); margin-bottom: 20px; }
        .lp-plan-features {
          list-style: none;
          display: flex;
          flex-direction: column;
          gap: 10px;
          margin-bottom: 28px;
          padding: 0;
        }
        .lp-plan-features li {
          display: flex;
          align-items: center;
          gap: 10px;
          font-size: 0.8rem;
          color: var(--text-muted);
        }
        .lp-plan-features li::before {
          content: '♦';
          font-size: 0.5rem;
          color: var(--primary);
          flex-shrink: 0;
        }
        .lp-plan-btn {
          width: 100%;
          padding: 12px;
          border-radius: 8px;
          font-family: 'DM Sans', sans-serif;
          font-size: 0.85rem;
          font-weight: 700;
          cursor: pointer;
          transition: all 0.2s;
        }
        .lp-plan-btn.primary {
          background: linear-gradient(135deg, var(--primary), var(--primary2));
          border: none;
          color: white;
          box-shadow: 0 4px 16px rgba(162,92,246,0.3);
        }
        .lp-plan-btn.primary:hover { opacity: 0.9; transform: translateY(-1px); }
        .lp-plan-btn.outline {
          background: transparent;
          border: 1px solid var(--border);
          color: var(--text-muted);
        }
        .lp-plan-btn.outline:hover { border-color: var(--border-accent); color: var(--text); }

        /* FOOTER */
        .lp-footer {
          position: relative;
          z-index: 1;
          background: rgba(0,0,0,0.3);
          border-top: 1px solid var(--border);
          padding: 60px 80px 32px;
        }
        [data-theme="light"] .lp-footer { background: rgba(124,58,237,0.04); }
        .lp-footer-grid {
          display: grid;
          grid-template-columns: 2fr 1fr 1fr 1fr;
          gap: 48px;
          max-width: 1280px;
          margin: 0 auto 48px;
        }
        .lp-footer-logo {
          font-family: 'Syne', sans-serif;
          font-size: 1.2rem;
          font-weight: 800;
          color: var(--text);
          display: flex;
          align-items: center;
          gap: 6px;
          margin-bottom: 14px;
        }
        .lp-footer-tagline { font-size: 0.8rem; color: var(--text-muted); line-height: 1.65; max-width: 240px; font-weight: 300; }
        .lp-footer-col h4 {
          font-size: 0.72rem;
          font-weight: 700;
          color: var(--text);
          text-transform: uppercase;
          letter-spacing: 1px;
          margin-bottom: 16px;
        }
        .lp-footer-col a {
          display: block;
          font-size: 0.8rem;
          color: var(--text-muted);
          text-decoration: none;
          margin-bottom: 10px;
          transition: color 0.2s;
        }
        .lp-footer-col a:hover { color: var(--primary); }
        .lp-footer-bottom {
          max-width: 1280px;
          margin: 0 auto;
          padding-top: 24px;
          border-top: 1px solid var(--border);
          display: flex;
          justify-content: space-between;
          align-items: center;
        }
        .lp-footer-copy { font-size: 0.75rem; color: var(--text-muted); }
        .lp-footer-badges { display: flex; gap: 12px; }
        .lp-footer-badge {
          font-size: 0.68rem;
          color: var(--text-muted);
          border: 1px solid var(--border);
          padding: 3px 10px;
          border-radius: 999px;
        }

        @media (max-width: 960px) {
          .lp-hero { grid-template-columns: 1fr; padding: 100px 32px 60px; gap: 40px; }
          .lp-features-grid, .lp-pricing-grid { grid-template-columns: 1fr; }
          .lp-nav { padding: 0 24px; }
          .lp-nav-links { display: none; }
          .lp-section { padding: 60px 32px; }
          .lp-partners-strip { padding: 24px 32px; gap: 24px; flex-wrap: wrap; }
          .lp-footer { padding: 48px 32px 24px; }
          .lp-footer-grid { grid-template-columns: 1fr 1fr; gap: 32px; }
        }
      `}</style>

      <div className="lp-root">
        <div className="lp-atmosphere">
          <div className="lp-orb lp-orb-1" />
          <div className="lp-orb lp-orb-2" />
          <div className="lp-orb lp-orb-3" />
        </div>

        {/* NAV */}
        <nav className="lp-nav">
          <button className="lp-nav-logo">
            <span className="lp-diamond">♦</span> Pulse
          </button>
          <div className="lp-nav-links">
            <a href="#features">Features</a>
            <a href="#how">How it Works</a>
            <a href="#pricing">Pricing</a>
            <a href="#about">About</a>
          </div>
          <div className="lp-nav-actions">
            <button className="lp-theme-toggle" onClick={toggleTheme} title="Toggle theme">
              {isDark ? "○" : "●"}
            </button>
            {walletConnected && (
              <button className="lp-btn-ghost" onClick={onEnterApp}>
                Open Dashboard
              </button>
            )}
            {walletConnected ? (
              <button className="lp-btn-connected" onClick={handleDisconnect}>
                ♦ {wallet.address?.slice(0, 4)}...{wallet.address?.slice(-4)}
              </button>
            ) : (
              <button
                className="lp-btn-primary"
                onClick={handleConnectWallet}
                disabled={isLoading}
              >
                {isLoading ? "Connecting..." : "Connect Wallet"}
              </button>
            )}
          </div>
        </nav>

        {/* HERO */}
        <section>
          <div className="lp-hero">
            <div className="lp-hero-left">
              <div className="lp-hero-tag">
                <span className="lp-dot" />
                Built on Stellar / Soroban
              </div>
              <h1 className="lp-hero-title">
                Trustless <span className="lp-hl">Crypto</span><br />
                Inheritance Protocol
              </h1>
              <p className="lp-hero-sub">
                Secure your digital assets for your loved ones. Automate inheritance,
                manage beneficiaries and prove liveness — all on-chain with Stellar.
              </p>
              <div className="lp-hero-btns">
                <button
                  className="lp-btn-primary"
                  onClick={walletConnected ? onEnterApp : handleConnectWallet}
                  disabled={isLoading}
                >
                  {walletConnected
                    ? "Open Dashboard →"
                    : isLoading
                    ? "Connecting..."
                    : "Connect Wallet"}
                </button>
                {walletConnected && (
                  <button className="lp-btn-ghost" onClick={onEnterApp}>
                    Go to Dashboard
                  </button>
                )}
              </div>
              {wallet.status === "missing" && (
                <p style={{ marginTop: 12, fontSize: "0.82rem", color: "#f87171" }}>
                  Freighter wallet extension not found.{" "}
                  <a
                    href="https://freighter.app"
                    target="_blank"
                    rel="noreferrer"
                    style={{ color: "var(--primary)" }}
                  >
                    Install it here →
                  </a>
                </p>
              )}
              {wallet.status === "error" && (
                <p style={{ marginTop: 12, fontSize: "0.82rem", color: "#f87171" }}>
                  {wallet.error}
                </p>
              )}
              <div className="lp-hero-stats">
                <div>
                  <div className="lp-stat-val">$2.4B</div>
                  <div className="lp-stat-lbl">Assets Secured</div>
                </div>
                <div>
                  <div className="lp-stat-val">48K</div>
                  <div className="lp-stat-lbl">Vaults Created</div>
                </div>
                <div>
                  <div className="lp-stat-val">99.9%</div>
                  <div className="lp-stat-lbl">Uptime</div>
                </div>
              </div>
            </div>

            <div className="lp-hero-right">
              <div className="lp-wallet-card">
                {!walletConnected ? (
                  <>
                    <div className="lp-w-icon">♦</div>
                    <div className="lp-wc-title">Connect Your Wallet</div>
                    <p className="lp-wc-desc">
                      Link your Freighter wallet to access your full vault, track assets
                      and configure your inheritance plan on Stellar.
                    </p>
                    <button
                      className="lp-wc-btn"
                      onClick={handleConnectWallet}
                      disabled={isLoading}
                    >
                      {isLoading ? "Connecting..." : "Connect with Freighter"}
                    </button>
                    {wallet.status === "missing" && (
                      <p style={{ marginTop: 10, fontSize: "0.75rem", color: "#f87171", textAlign: "center" }}>
                        Freighter not installed.{" "}
                        <a href="https://freighter.app" target="_blank" rel="noreferrer" style={{ color: "var(--primary)" }}>
                          Download here
                        </a>
                      </p>
                    )}
                    {wallet.status === "error" && (
                      <p style={{ marginTop: 10, fontSize: "0.75rem", color: "#f87171", textAlign: "center" }}>
                        {wallet.error}
                      </p>
                    )}
                  </>
                ) : (
                  <>
                    <div className="lp-c-top">
                      <div className="lp-c-badge">
                        <span className="lp-c-dot" />
                        Connected
                      </div>
                      <div className="lp-c-addr">
                        {wallet.address?.slice(0, 6)}...{wallet.address?.slice(-4)}
                      </div>
                    </div>
                    <div className="lp-c-balance-label">Wallet Address</div>
                    <div className="lp-c-balance" style={{ fontSize: "1rem", letterSpacing: 0, wordBreak: "break-all" }}>
                      {wallet.address}
                    </div>
                    <div className="lp-c-change" style={{ marginTop: 8 }}>
                      ♦ Network: {wallet.network || "unknown"}
                    </div>
                    <div style={{ marginTop: 20 }}>
                      <button className="lp-btn-dashboard" onClick={onEnterApp}>
                        <span>♦</span> Open Dashboard
                      </button>
                      <button className="lp-btn-disconnect" onClick={handleDisconnect}>
                        <span>♦</span> Disconnect Wallet
                      </button>
                    </div>
                  </>
                )}
              </div>
            </div>
          </div>
        </section>

        {/* PARTNERS */}
        <div className="lp-partners-strip">
          <span className="lp-partners-label">Powered by</span>
          <span className="lp-partner">STELLAR</span>
          <span className="lp-partner">SOROBAN</span>
          <span className="lp-partner">Trustless Work</span>
          <span className="lp-partner">Freighter</span>
          <span className="lp-partner">IPFS</span>
        </div>

        {/* FEATURES */}
        <div className="lp-section" id="features">
          <div className="lp-section-pill">♦ Features</div>
          <h2 className="lp-section-title">
            Everything your estate needs,{" "}
            <span className="lp-hl">nothing left to chance</span>
          </h2>
          <p className="lp-section-sub">
            Achieve peace of mind with tools designed to secure, automate and
            personalize your crypto inheritance plan.
          </p>
          <div className="lp-features-grid">
            <div className="lp-feature-card">
              <div className="lp-feature-icon">♦</div>
              <div className="lp-feature-title">Automated Distribution</div>
              <div className="lp-feature-desc">
                Define beneficiaries and allocations once. Assets distribute
                automatically when liveness conditions aren't met.
              </div>
            </div>
            <div className="lp-feature-card">
              <div className="lp-feature-icon">♣</div>
              <div className="lp-feature-title">Liveness Proof</div>
              <div className="lp-feature-desc">
                Regular on-chain verification signals keep your vault active.
                Miss a check-in and the inheritance protocol activates.
              </div>
            </div>
            <div className="lp-feature-card">
              <div className="lp-feature-icon">♠</div>
              <div className="lp-feature-title">Trustless Escrow</div>
              <div className="lp-feature-desc">
                Powered by Soroban smart contracts — no intermediaries, no
                single point of failure. Your keys, your rules.
              </div>
            </div>
          </div>
        </div>

        {/* PRICING */}
        <div className="lp-section" id="pricing">
          <div className="lp-section-pill">♦ Pricing</div>
          <h2 className="lp-section-title">
            Simple, <span className="lp-hl">transparent</span> pricing
          </h2>
          <p className="lp-section-sub">
            Try it free on testnet. Upgrade when you're ready for mainnet. No hidden fees.
          </p>
          <div className="lp-pricing-grid">
            <div className="lp-price-card">
              <div className="lp-plan-name">Starter</div>
              <div className="lp-plan-price"><sup>$</sup>0<span>/mo</span></div>
              <div className="lp-plan-desc">For individuals exploring trustless inheritance on testnet.</div>
              <div className="lp-plan-divider" />
              <ul className="lp-plan-features">
                <li>1 vault</li>
                <li>Up to 3 beneficiaries</li>
                <li>Basic liveness checks</li>
                <li>Community support</li>
              </ul>
              <button className="lp-plan-btn outline" onClick={onEnterApp}>Get Started</button>
            </div>
            <div className="lp-price-card featured">
              <div className="lp-plan-name">Pro</div>
              <div className="lp-plan-price"><sup>$</sup>19<span>/mo</span></div>
              <div className="lp-plan-desc">For serious holders who need full control and automation.</div>
              <div className="lp-plan-divider" />
              <ul className="lp-plan-features">
                <li>Unlimited vaults</li>
                <li>Unlimited beneficiaries</li>
                <li>Advanced liveness signals</li>
                <li>Priority support</li>
                <li>Full activity audit log</li>
              </ul>
              <button className="lp-plan-btn primary" onClick={onEnterApp}>Start Free Trial</button>
            </div>
            <div className="lp-price-card">
              <div className="lp-plan-name">Enterprise</div>
              <div className="lp-plan-price">Custom</div>
              <div className="lp-plan-desc">For funds and institutions managing complex digital estates.</div>
              <div className="lp-plan-divider" />
              <ul className="lp-plan-features">
                <li>Everything in Pro</li>
                <li>Multi-sig support</li>
                <li>Custom integrations</li>
                <li>Dedicated account manager</li>
              </ul>
              <button className="lp-plan-btn outline">Contact Sales</button>
            </div>
          </div>
        </div>

        {/* FOOTER */}
        <footer className="lp-footer">
          <div className="lp-footer-grid">
            <div>
              <div className="lp-footer-logo">
                <span className="lp-diamond">♦</span> Pulse
              </div>
              <div className="lp-footer-tagline">
                Trustless crypto inheritance built on Stellar. Secure your digital
                legacy for the people you love.
              </div>
            </div>
            <div className="lp-footer-col">
              <h4>Product</h4>
              <a href="#features">Features</a>
              <a href="#pricing">Pricing</a>
              <a href="#">Changelog</a>
              <a href="#">Roadmap</a>
            </div>
            <div className="lp-footer-col">
              <h4>Company</h4>
              <a href="#">About</a>
              <a href="#">Blog</a>
              <a href="#">Careers</a>
              <a href="#">Press</a>
            </div>
            <div className="lp-footer-col">
              <h4>Legal</h4>
              <a href="#">Privacy</a>
              <a href="#">Terms</a>
              <a href="#">Security</a>
              <a href="#">Cookies</a>
            </div>
          </div>
          <div className="lp-footer-bottom">
            <div className="lp-footer-copy">♦ 2025 Pulse Protocol. All rights reserved.</div>
            <div className="lp-footer-badges">
              <span className="lp-footer-badge">Stellar Testnet</span>
              <span className="lp-footer-badge">Soroban Smart Contracts</span>
              <span className="lp-footer-badge">Non-Custodial</span>
            </div>
          </div>
        </footer>
      </div>
    </div>
  );
}