/// BiometricSetupScreen — Registers fingerprint via BiometricPrompt.
/// Part of the SimpleOnboardingNavigator.

import React, { useState, useCallback } from "react";
import {
  View,
  Text,
  TouchableOpacity,
  ActivityIndicator,
  StyleSheet,
} from "react-native";
import { checkBiometricAvailability, authenticateWithBiometric } from "../services/biometrics/fingerprint";
import { useAuthStore } from "../stores/authStore";

type Props = {
  onSetupComplete: () => void;
};

type ScreenState = "idle" | "checking" | "authenticating" | "done" | "error" | "unavailable";

export default function BiometricSetupScreen({ onSetupComplete }: Props) {
  const [state, setState] = useState<ScreenState>("idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const setBiometricSetup = useAuthStore((s) => s.setBiometricSetup);

  const handleSetup = useCallback(async () => {
    setState("checking");
    setErrorMessage(null);

    try {
      const available = await checkBiometricAvailability();
      if (!available) {
        setState("unavailable");
        return;
      }

      setState("authenticating");
      const result = await authenticateWithBiometric("Register your fingerprint for Pulse Protocol");

      if (result.success) {
        setBiometricSetup(true);
        setState("done");
      } else {
        setState("error");
        setErrorMessage("Biometric authentication failed. Please try again.");
      }
    } catch (err: unknown) {
      setState("error");
      const message = err instanceof Error ? err.message : "Setup failed";
      setErrorMessage(message);
    }
  }, [setBiometricSetup]);

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Biometric Setup</Text>

      {state === "idle" && (
        <View style={styles.content}>
          <Text style={styles.icon}>&#128274;</Text>
          <Text style={styles.description}>
            Set up your fingerprint for proof-of-life verification. Your
            biometric data never leaves this device.
          </Text>
          <TouchableOpacity style={styles.button} onPress={handleSetup}>
            <Text style={styles.buttonText}>Setup Fingerprint</Text>
          </TouchableOpacity>
        </View>
      )}

      {(state === "checking" || state === "authenticating") && (
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#6200EE" />
          <Text style={styles.loadingText}>
            {state === "checking"
              ? "Checking biometric availability..."
              : "Place your finger on the sensor..."}
          </Text>
        </View>
      )}

      {state === "done" && (
        <View style={styles.content}>
          <Text style={styles.successIcon}>&#10003;</Text>
          <Text style={styles.successTitle}>Fingerprint Registered</Text>
          <Text style={styles.successDesc}>
            Your fingerprint is ready for proof-of-life verification.
          </Text>
          <TouchableOpacity style={styles.button} onPress={onSetupComplete}>
            <Text style={styles.buttonText}>Go to Dashboard</Text>
          </TouchableOpacity>
        </View>
      )}

      {state === "unavailable" && (
        <View style={styles.content}>
          <Text style={styles.warningText}>
            Biometric authentication is not available on this device. You can
            still use the app with manual verification.
          </Text>
          <TouchableOpacity style={styles.button} onPress={() => {
            setBiometricSetup(true);
            onSetupComplete();
          }}>
            <Text style={styles.buttonText}>Skip & Continue</Text>
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
    backgroundColor: "#FAFAFA",
    alignItems: "center",
    justifyContent: "center",
  },
  title: {
    fontSize: 24,
    fontWeight: "bold",
    color: "#212121",
    marginBottom: 32,
  },
  content: {
    alignItems: "center",
    width: "100%",
  },
  icon: {
    fontSize: 64,
    marginBottom: 24,
  },
  description: {
    fontSize: 16,
    color: "#757575",
    textAlign: "center",
    lineHeight: 24,
    marginBottom: 40,
    paddingHorizontal: 16,
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
    color: "#757575",
  },
  successIcon: {
    fontSize: 64,
    color: "#4CAF50",
    marginBottom: 16,
  },
  successTitle: {
    fontSize: 20,
    fontWeight: "600",
    color: "#4CAF50",
    marginBottom: 8,
  },
  successDesc: {
    fontSize: 14,
    color: "#757575",
    textAlign: "center",
    marginBottom: 32,
  },
  warningText: {
    fontSize: 16,
    color: "#FF9800",
    textAlign: "center",
    lineHeight: 24,
    marginBottom: 32,
  },
  errorText: {
    fontSize: 16,
    color: "#F44336",
    textAlign: "center",
    marginBottom: 24,
  },
});
