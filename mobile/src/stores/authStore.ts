/// Zustand store for authentication state with AsyncStorage persistence.
import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import AsyncStorage from "@react-native-async-storage/async-storage";

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

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
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
    }),
    {
      name: "pulse-auth-storage",
      storage: createJSONStorage(() => AsyncStorage),
    }
  )
);
