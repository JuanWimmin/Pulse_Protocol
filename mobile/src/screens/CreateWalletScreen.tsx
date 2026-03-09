/// CreateWalletScreen — Generates a Stellar keypair and stores secret in keychain.
/// After creation, authenticates with the backend and navigates to onboarding.

import React, { useState, useCallback } from "react";
import {
  View,
  Text,
  TouchableOpacity,
  ActivityIndicator,
  StyleSheet,
} from "react-native";
import { Keypair } from "@stellar/stellar-sdk";
import * as Keychain from "react-native-keychain";
import { useAuthStore } from "../stores/authStore";
import { authenticateWithBackend } from "../services/auth";

type Props = {
  onWalletCreated: () => void;
};

type ScreenState = "idle" | "generating" | "done" | "error";

export default function CreateWalletScreen({ onWalletCreated }: Props) {
  const [state, setState] = useState<ScreenState>("idle");
  const [address, setAddress] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const setWallet = useAuthStore((s) => s.setWallet);
  const setSession = useAuthStore((s) => s.setSession);

  const handleCreate = useCallback(async () => {
    setState("generating");
    setErrorMessage(null);

    try {
      // Generate Stellar keypair
      const keypair = Keypair.random();
      const publicKey = keypair.publicKey();
      await Keychain.setGenericPassword("stellar_secret", keypair.secret());

      setAddress(publicKey);
      setWallet(publicKey);

      // Authenticate with backend
      const authResult = await authenticateWithBackend(keypair);
      setSession(authResult.token);

      setState("done");
    } catch (err: unknown) {
      setState("error");
      const message = err instanceof Error ? err.message : "Failed to create wallet";
      setErrorMessage(message);
    }
  }, [setWallet, setSession]);

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Create Your Wallet</Text>

      {state === "idle" && (
        <View style={styles.content}>
          <Text style={styles.description}>
            A new Stellar wallet will be generated for you. Your private key
            will be stored securely on this device.
          </Text>
          <TouchableOpacity style={styles.button} onPress={handleCreate}>
            <Text style={styles.buttonText}>Generate Wallet</Text>
          </TouchableOpacity>
        </View>
      )}

      {state === "generating" && (
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#6200EE" />
          <Text style={styles.loadingText}>Generating keypair...</Text>
        </View>
      )}

      {state === "done" && address && (
        <View style={styles.content}>
          <Text style={styles.successTitle}>Wallet Created</Text>
          <View style={styles.addressCard}>
            <Text style={styles.addressLabel}>Your Stellar Address</Text>
            <Text style={styles.addressValue} numberOfLines={2}>
              {address}
            </Text>
          </View>
          <Text style={styles.warningText}>
            Your private key is stored securely on this device. Do not share it.
          </Text>
          <TouchableOpacity
            style={styles.button}
            onPress={onWalletCreated}
          >
            <Text style={styles.buttonText}>Continue</Text>
          </TouchableOpacity>
        </View>
      )}

      {state === "error" && (
        <View style={styles.content}>
          <Text style={styles.errorText}>{errorMessage}</Text>
          <TouchableOpacity
            style={[styles.button, styles.retryButton]}
            onPress={() => setState("idle")}
          >
            <Text style={styles.buttonText}>Try Again</Text>
          </TouchableOpacity>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 24,
    backgroundColor: "#1A1A2E",
    alignItems: "center",
    justifyContent: "center",
  },
  title: {
    fontSize: 24,
    fontWeight: "bold",
    color: "#FFFFFF",
    marginBottom: 32,
  },
  content: {
    alignItems: "center",
    width: "100%",
  },
  description: {
    fontSize: 16,
    color: "#CCCCCC",
    textAlign: "center",
    lineHeight: 24,
    marginBottom: 40,
  },
  button: {
    backgroundColor: "#6200EE",
    paddingHorizontal: 48,
    paddingVertical: 16,
    borderRadius: 28,
    elevation: 2,
  },
  buttonText: {
    color: "#FFFFFF",
    fontSize: 18,
    fontWeight: "600",
  },
  retryButton: {
    backgroundColor: "#757575",
  },
  loadingContainer: {
    alignItems: "center",
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: "#AAAAAA",
  },
  successTitle: {
    fontSize: 20,
    fontWeight: "600",
    color: "#4CAF50",
    marginBottom: 24,
  },
  addressCard: {
    backgroundColor: "#2A2A3E",
    padding: 20,
    borderRadius: 12,
    width: "100%",
    marginBottom: 16,
  },
  addressLabel: {
    fontSize: 12,
    color: "#AAAAAA",
    marginBottom: 8,
  },
  addressValue: {
    fontSize: 13,
    fontFamily: "monospace",
    color: "#BB86FC",
  },
  warningText: {
    fontSize: 13,
    color: "#FF9800",
    textAlign: "center",
    marginBottom: 32,
  },
  errorText: {
    fontSize: 16,
    color: "#F44336",
    textAlign: "center",
    marginBottom: 24,
  },
});
