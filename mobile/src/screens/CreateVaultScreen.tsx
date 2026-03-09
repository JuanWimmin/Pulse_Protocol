/// CreateVaultScreen — Create a new vault with token selection and initial deposit.
/// Calls CREATE_VAULT + optional DEPOSIT mutations.

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
import { CREATE_VAULT, DEPOSIT } from "../services/graphql/mutations";

type Props = {
  onVaultCreated: (vaultId: string) => void;
  onGoBack: () => void;
};

export default function CreateVaultScreen({ onVaultCreated, onGoBack }: Props) {
  const [token, setToken] = useState("native");
  const [initialDeposit, setInitialDeposit] = useState("");
  const [loading, setLoading] = useState(false);

  const [createVault] = useMutation(CREATE_VAULT);
  const [deposit] = useMutation(DEPOSIT);

  const handleCreate = useCallback(async () => {
    setLoading(true);
    try {
      const { data } = await createVault({
        variables: { input: { token, initialDeposit: initialDeposit || undefined } },
      });

      const vault = data.createVault;

      if (initialDeposit && Number(initialDeposit) > 0) {
        await deposit({
          variables: {
            vaultId: vault.id,
            amount: initialDeposit,
            token,
          },
        });
      }

      onVaultCreated(vault.id);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Failed to create vault";
      Alert.alert("Error", message);
    } finally {
      setLoading(false);
    }
  }, [token, initialDeposit, createVault, deposit, onVaultCreated]);

  return (
    <View style={styles.container}>
      <TouchableOpacity style={styles.backButton} onPress={onGoBack}>
        <Text style={styles.backText}>Back</Text>
      </TouchableOpacity>

      <Text style={styles.title}>Create Vault</Text>

      <View style={styles.form}>
        <Text style={styles.label}>Token</Text>
        <View style={styles.tokenSelector}>
          {["native", "USDC"].map((t) => (
            <TouchableOpacity
              key={t}
              style={[styles.tokenOption, token === t && styles.tokenSelected]}
              onPress={() => setToken(t)}
            >
              <Text
                style={[
                  styles.tokenText,
                  token === t && styles.tokenTextSelected,
                ]}
              >
                {t === "native" ? "XLM" : t}
              </Text>
            </TouchableOpacity>
          ))}
        </View>

        <Text style={styles.label}>Initial Deposit (optional)</Text>
        <TextInput
          style={styles.input}
          placeholder="Amount in stroops"
          placeholderTextColor="#BDBDBD"
          keyboardType="numeric"
          value={initialDeposit}
          onChangeText={setInitialDeposit}
        />
        <Text style={styles.hint}>1 XLM = 10,000,000 stroops</Text>

        <TouchableOpacity
          style={[styles.createButton, loading && styles.disabledButton]}
          onPress={handleCreate}
          disabled={loading}
        >
          {loading ? (
            <ActivityIndicator size="small" color="#FFFFFF" />
          ) : (
            <Text style={styles.createButtonText}>Create Vault</Text>
          )}
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 24,
    backgroundColor: "#FAFAFA",
  },
  backButton: {
    marginBottom: 16,
  },
  backText: {
    fontSize: 16,
    color: "#6200EE",
  },
  title: {
    fontSize: 24,
    fontWeight: "bold",
    color: "#212121",
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
  tokenSelector: {
    flexDirection: "row",
    marginBottom: 24,
  },
  tokenOption: {
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 20,
    backgroundColor: "#E0E0E0",
    marginRight: 12,
  },
  tokenSelected: {
    backgroundColor: "#6200EE",
  },
  tokenText: {
    fontSize: 14,
    fontWeight: "600",
    color: "#757575",
  },
  tokenTextSelected: {
    color: "#FFFFFF",
  },
  input: {
    backgroundColor: "#FFFFFF",
    borderWidth: 1,
    borderColor: "#E0E0E0",
    borderRadius: 12,
    padding: 16,
    fontSize: 16,
    color: "#212121",
    marginBottom: 8,
  },
  hint: {
    fontSize: 12,
    color: "#BDBDBD",
    marginBottom: 32,
  },
  createButton: {
    backgroundColor: "#6200EE",
    paddingVertical: 16,
    borderRadius: 28,
    alignItems: "center",
    elevation: 2,
  },
  disabledButton: {
    opacity: 0.7,
  },
  createButtonText: {
    color: "#FFFFFF",
    fontSize: 18,
    fontWeight: "600",
  },
});
