/// Zustand store for vault state.
import { create } from "zustand";
import { Vault } from "../types/vault";
import { VaultStatus } from "../types/verification";

interface VaultState {
  currentVault: Vault | null;
  vaults: Vault[];
  livenessScore: number | null;
  lastVerified: string | null;

  setCurrentVault: (vault: Vault | null) => void;
  setVaults: (vaults: Vault[]) => void;
  updateVaultStatus: (vaultId: string, status: VaultStatus) => void;
  setLiveness: (score: number, lastVerified: string) => void;
  clear: () => void;
}

export const useVaultStore = create<VaultState>((set) => ({
  currentVault: null,
  vaults: [],
  livenessScore: null,
  lastVerified: null,

  setCurrentVault: (vault) => set({ currentVault: vault }),

  setVaults: (vaults) =>
    set({
      vaults,
      currentVault: vaults.length > 0 ? vaults[0] : null,
    }),

  updateVaultStatus: (vaultId, status) =>
    set((state) => ({
      vaults: state.vaults.map((v) =>
        v.id === vaultId ? { ...v, status } : v
      ),
      currentVault:
        state.currentVault?.id === vaultId
          ? { ...state.currentVault, status }
          : state.currentVault,
    })),

  setLiveness: (score, lastVerified) =>
    set({ livenessScore: score, lastVerified }),

  clear: () =>
    set({
      currentVault: null,
      vaults: [],
      livenessScore: null,
      lastVerified: null,
    }),
}));
