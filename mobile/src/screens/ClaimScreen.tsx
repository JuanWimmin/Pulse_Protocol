/// ClaimScreen — Beneficiary claims inheritance from a triggered vault.
/// Shows eligible vaults and allows claiming with result display.

import React, { useState, useCallback } from "react";
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
  StyleSheet,
  Alert,
} from "react-native";
import { useMutation } from "@apollo/client";
import { CLAIM_INHERITANCE } from "../services/graphql/mutations";

type Props = {
  onGoBack?: () => void;
};

interface ClaimResultData {
  success: boolean;
  amountReceived: string;
  txHash: string | null;
}

export default function ClaimScreen({ onGoBack }: Props) {
  const [vaultId, setVaultId] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ClaimResultData | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [claimInheritance] = useMutation(CLAIM_INHERITANCE);

  const handleClaim = useCallback(async () => {
    if (!vaultId.trim()) {
      Alert.alert("Error", "Please enter a vault ID");
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const { data } = await claimInheritance({
        variables: { vaultId: vaultId.trim() },
      });

      setResult(data.claimInheritance);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Claim failed";
      setError(message);
    } finally {
      setLoading(false);
    }
  }, [vaultId, claimInheritance]);

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Claim Inheritance</Text>
      <Text style={styles.subtitle}>
        If you are a beneficiary of a triggered vault, you can claim your
        allocated share here.
      </Text>

      {!result && (
        <View style={styles.form}>
          <Text style={styles.label}>Vault ID</Text>
          <TextInput
            style={styles.input}
            placeholder="Enter the vault ID"
            placeholderTextColor="#BDBDBD"
            value={vaultId}
            onChangeText={setVaultId}
          />

          <TouchableOpacity
            style={[styles.claimButton, loading && styles.disabledButton]}
            onPress={handleClaim}
            disabled={loading}
          >
            {loading ? (
              <ActivityIndicator size="small" color="#FFFFFF" />
            ) : (
              <Text style={styles.claimButtonText}>Claim Inheritance</Text>
            )}
          </TouchableOpacity>
        </View>
      )}

      {result && (
        <View style={styles.resultCard}>
          <Text style={styles.resultTitle}>
            {result.success ? "Claim Successful" : "Claim Failed"}
          </Text>

          {result.success && (
            <>
              <Text style={styles.amountLabel}>Amount Received</Text>
              <Text style={styles.amountValue}>{result.amountReceived}</Text>

              {result.txHash && (
                <View style={styles.txContainer}>
                  <Text style={styles.txLabel}>Transaction Hash</Text>
                  <Text style={styles.txHash} numberOfLines={1}>
                    {result.txHash}
                  </Text>
                </View>
              )}
            </>
          )}

          <TouchableOpacity
            style={styles.resetButton}
            onPress={() => {
              setResult(null);
              setVaultId("");
            }}
          >
            <Text style={styles.resetButtonText}>Claim Another</Text>
          </TouchableOpacity>
        </View>
      )}

      {error && (
        <View style={styles.errorContainer}>
          <Text style={styles.errorText}>{error}</Text>
          <TouchableOpacity
            style={styles.retryButton}
            onPress={() => setError(null)}
          >
            <Text style={styles.retryButtonText}>Try Again</Text>
          </TouchableOpacity>
        </View>
      )}

      {onGoBack && (
        <TouchableOpacity style={styles.backButton} onPress={onGoBack}>
          <Text style={styles.backButtonText}>Back to Dashboard</Text>
        </TouchableOpacity>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 24,
    backgroundColor: "#FAFAFA",
    justifyContent: "center",
  },
  title: {
    fontSize: 24,
    fontWeight: "bold",
    color: "#212121",
    marginBottom: 8,
    textAlign: "center",
  },
  subtitle: {
    fontSize: 14,
    color: "#757575",
    textAlign: "center",
    lineHeight: 20,
    marginBottom: 32,
  },
  form: {
    width: "100%",
  },
  label: {
    fontSize: 14,
    fontWeight: "600",
    color: "#757575",
    marginBottom: 8,
  },
  input: {
    backgroundColor: "#FFFFFF",
    borderWidth: 1,
    borderColor: "#E0E0E0",
    borderRadius: 12,
    padding: 16,
    fontSize: 14,
    color: "#212121",
    marginBottom: 24,
  },
  claimButton: {
    backgroundColor: "#4CAF50",
    paddingVertical: 16,
    borderRadius: 28,
    alignItems: "center",
    elevation: 2,
  },
  disabledButton: {
    opacity: 0.7,
  },
  claimButtonText: {
    color: "#FFFFFF",
    fontSize: 18,
    fontWeight: "600",
  },
  resultCard: {
    backgroundColor: "#FFFFFF",
    padding: 24,
    borderRadius: 16,
    elevation: 2,
    alignItems: "center",
  },
  resultTitle: {
    fontSize: 20,
    fontWeight: "bold",
    color: "#4CAF50",
    marginBottom: 16,
  },
  amountLabel: {
    fontSize: 12,
    color: "#757575",
    marginBottom: 4,
  },
  amountValue: {
    fontSize: 32,
    fontWeight: "bold",
    color: "#212121",
    marginBottom: 16,
  },
  txContainer: {
    width: "100%",
    backgroundColor: "#F5F5F5",
    padding: 12,
    borderRadius: 8,
    marginBottom: 20,
  },
  txLabel: {
    fontSize: 11,
    color: "#757575",
    marginBottom: 4,
  },
  txHash: {
    fontSize: 12,
    fontFamily: "monospace",
    color: "#212121",
  },
  resetButton: {
    backgroundColor: "#757575",
    paddingHorizontal: 32,
    paddingVertical: 12,
    borderRadius: 24,
  },
  resetButtonText: {
    color: "#FFFFFF",
    fontSize: 14,
    fontWeight: "600",
  },
  errorContainer: {
    alignItems: "center",
    marginTop: 16,
  },
  errorText: {
    fontSize: 16,
    color: "#F44336",
    textAlign: "center",
    marginBottom: 16,
  },
  retryButton: {
    backgroundColor: "#F44336",
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 20,
  },
  retryButtonText: {
    color: "#FFFFFF",
    fontSize: 14,
    fontWeight: "600",
  },
  backButton: {
    position: "absolute",
    bottom: 32,
    alignSelf: "center",
  },
  backButtonText: {
    fontSize: 14,
    color: "#6200EE",
  },
});
