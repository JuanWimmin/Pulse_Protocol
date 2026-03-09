/// Zustand store for authentication state.
import { create } from "zustand";

interface AuthState {
  stellarAddress: string | null;
  sessionToken: string | null;
  isAuthenticated: boolean;
  biometricSetupComplete: boolean;

  setWallet: (address: string) => void;
  setSession: (token: string) => void;
  setBiometricSetup: (done: boolean) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  stellarAddress: null,
  sessionToken: null,
  isAuthenticated: false,
  biometricSetupComplete: false,

  setWallet: (address) =>
    set({ stellarAddress: address }),

  setSession: (token) =>
    set({ sessionToken: token, isAuthenticated: true }),

  setBiometricSetup: (done) =>
    set({ biometricSetupComplete: done }),

  logout: () =>
    set({
      stellarAddress: null,
      sessionToken: null,
      isAuthenticated: false,
      biometricSetupComplete: false,
    }),
}));
