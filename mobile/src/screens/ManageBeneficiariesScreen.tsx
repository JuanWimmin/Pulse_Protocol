/// ManageBeneficiariesScreen — Add/edit beneficiaries with percentage allocation.
/// Validates percentages sum to exactly 100% (10000 basis points).
/// Calls SET_BENEFICIARIES mutation.

import React, { useState, useCallback } from "react";
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  ScrollView,
  ActivityIndicator,
  StyleSheet,
  Alert,
} from "react-native";
import { useMutation } from "@apollo/client";
import { SET_BENEFICIARIES } from "../services/graphql/mutations";

type Props = {
  vaultId: string;
  onGoBack: () => void;
};

interface BeneficiaryEntry {
  address: string;
  percentage: string;
}

export default function ManageBeneficiariesScreen({ vaultId, onGoBack }: Props) {
  const [entries, setEntries] = useState<BeneficiaryEntry[]>([
    { address: "", percentage: "" },
  ]);
  const [loading, setLoading] = useState(false);

  const [setBeneficiaries] = useMutation(SET_BENEFICIARIES);

  const totalPercentage = entries.reduce(
    (sum, e) => sum + (Number(e.percentage) || 0),
    0
  );
  const isValid =
    totalPercentage === 10000 &&
    entries.every((e) => e.address.startsWith("G") && e.address.length === 56);

  const updateEntry = (index: number, field: keyof BeneficiaryEntry, value: string) => {
    const updated = [...entries];
    updated[index] = { ...updated[index], [field]: value };
    setEntries(updated);
  };

  const addEntry = () => {
    setEntries([...entries, { address: "", percentage: "" }]);
  };

  const removeEntry = (index: number) => {
    if (entries.length <= 1) return;
    setEntries(entries.filter((_, i) => i !== index));
  };

  const handleSave = useCallback(async () => {
    if (!isValid) {
      Alert.alert("Validation Error", "Percentages must sum to 10000 (100.00%)");
      return;
    }

    setLoading(true);
    try {
      await setBeneficiaries({
        variables: {
          vaultId,
          beneficiaries: entries.map((e) => ({
            address: e.address,
            percentage: Number(e.percentage),
          })),
        },
      });

      Alert.alert("Success", "Beneficiaries updated successfully");
      onGoBack();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Failed to set beneficiaries";
      Alert.alert("Error", message);
    } finally {
      setLoading(false);
    }
  }, [entries, isValid, vaultId, setBeneficiaries, onGoBack]);

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <TouchableOpacity onPress={onGoBack}>
        <Text style={styles.backLink}>Back</Text>
      </TouchableOpacity>

      <Text style={styles.title}>Manage Beneficiaries</Text>
      <Text style={styles.subtitle}>
        Allocate percentages to your beneficiaries. Total must equal 100.00%
        (10000 basis points).
      </Text>

      {entries.map((entry, index) => (
        <View key={index} style={styles.entryCard}>
          <View style={styles.entryHeader}>
            <Text style={styles.entryLabel}>Beneficiary #{index + 1}</Text>
            {entries.length > 1 && (
              <TouchableOpacity onPress={() => removeEntry(index)}>
                <Text style={styles.removeText}>Remove</Text>
              </TouchableOpacity>
            )}
          </View>

          <TextInput
            style={styles.input}
            placeholder="Stellar address (G...)"
            placeholderTextColor="#BDBDBD"
            value={entry.address}
            onChangeText={(v) => updateEntry(index, "address", v)}
            autoCapitalize="characters"
          />

          <TextInput
            style={styles.input}
            placeholder="Percentage (basis points, e.g. 5000 = 50%)"
            placeholderTextColor="#BDBDBD"
            keyboardType="numeric"
            value={entry.percentage}
            onChangeText={(v) => updateEntry(index, "percentage", v)}
          />
        </View>
      ))}

      <TouchableOpacity style={styles.addButton} onPress={addEntry}>
        <Text style={styles.addButtonText}>+ Add Beneficiary</Text>
      </TouchableOpacity>

      {/* Total indicator */}
      <View style={styles.totalCard}>
        <Text style={styles.totalLabel}>Total Allocation</Text>
        <Text
          style={[
            styles.totalValue,
            { color: totalPercentage === 10000 ? "#4CAF50" : "#F44336" },
          ]}
        >
          {(totalPercentage / 100).toFixed(2)}% ({totalPercentage} / 10000)
        </Text>
      </View>

      <TouchableOpacity
        style={[styles.saveButton, (!isValid || loading) && styles.disabledButton]}
        onPress={handleSave}
        disabled={!isValid || loading}
      >
        {loading ? (
          <ActivityIndicator size="small" color="#FFFFFF" />
        ) : (
          <Text style={styles.saveButtonText}>Save Beneficiaries</Text>
        )}
      </TouchableOpacity>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#FAFAFA",
  },
  content: {
    padding: 20,
  },
  backLink: {
    fontSize: 16,
    color: "#6200EE",
    marginBottom: 16,
  },
  title: {
    fontSize: 24,
    fontWeight: "bold",
    color: "#212121",
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    color: "#757575",
    marginBottom: 24,
    lineHeight: 20,
  },
  entryCard: {
    backgroundColor: "#FFFFFF",
    padding: 16,
    borderRadius: 12,
    elevation: 1,
    marginBottom: 12,
  },
  entryHeader: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: 12,
  },
  entryLabel: {
    fontSize: 14,
    fontWeight: "600",
    color: "#212121",
  },
  removeText: {
    fontSize: 13,
    color: "#F44336",
  },
  input: {
    backgroundColor: "#F5F5F5",
    borderRadius: 8,
    padding: 12,
    fontSize: 14,
    color: "#212121",
    marginBottom: 8,
  },
  addButton: {
    padding: 16,
    alignItems: "center",
    borderWidth: 1,
    borderColor: "#6200EE",
    borderRadius: 12,
    borderStyle: "dashed",
    marginBottom: 20,
  },
  addButtonText: {
    fontSize: 14,
    color: "#6200EE",
    fontWeight: "600",
  },
  totalCard: {
    backgroundColor: "#FFFFFF",
    padding: 16,
    borderRadius: 12,
    elevation: 1,
    alignItems: "center",
    marginBottom: 20,
  },
  totalLabel: {
    fontSize: 12,
    color: "#757575",
    marginBottom: 4,
  },
  totalValue: {
    fontSize: 20,
    fontWeight: "bold",
  },
  saveButton: {
    backgroundColor: "#6200EE",
    paddingVertical: 16,
    borderRadius: 28,
    alignItems: "center",
    elevation: 2,
  },
  disabledButton: {
    opacity: 0.5,
  },
  saveButtonText: {
    color: "#FFFFFF",
    fontSize: 18,
    fontWeight: "600",
  },
});
