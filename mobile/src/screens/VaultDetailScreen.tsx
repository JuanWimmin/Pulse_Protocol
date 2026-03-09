/// VaultDetailScreen — Shows vault balance, status, and beneficiary list.
/// Navigates to ManageBeneficiariesScreen for editing.

import React, { useState } from "react";
import {
  View,
  Text,
  TouchableOpacity,
  ScrollView,
  ActivityIndicator,
  StyleSheet,
  Alert,
} from "react-native";
import { useQuery, useMutation } from "@apollo/client";
import { VAULT_QUERY } from "../services/graphql/queries";
import { FORCE_TRANSITION } from "../services/graphql/mutations";

type Props = {
  vaultId: string;
  onNavigateManageBeneficiaries: (vaultId: string) => void;
  onGoBack: () => void;
};

const statusColor: Record<string, string> = {
  ACTIVE: "#4CAF50",
  ALERT: "#FF9800",
  GRACE_PERIOD: "#F44336",
  TRIGGERED: "#9E9E9E",
  DISTRIBUTED: "#607D8B",
};

const statusLabel: Record<string, string> = {
  ACTIVE: "Active",
  ALERT: "Alert",
  GRACE_PERIOD: "Grace Period",
  TRIGGERED: "Triggered",
  DISTRIBUTED: "Distributed",
};

export default function VaultDetailScreen({
  vaultId,
  onNavigateManageBeneficiaries,
  onGoBack,
}: Props) {
  const { data, loading, refetch } = useQuery(VAULT_QUERY, {
    variables: { id: vaultId },
  });
  const [forceTransition, { loading: transitioning }] = useMutation(FORCE_TRANSITION);
  const [transitionError, setTransitionError] = useState<string | null>(null);

  const vault = data?.vault;

  const handleForceTransition = async (newStatus: string) => {
    setTransitionError(null);
    try {
      await forceTransition({
        variables: { vaultId, newStatus },
      });
      await refetch();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Transition failed";
      setTransitionError(message);
      Alert.alert("Transition Failed", message);
    }
  };

  if (loading) {
    return (
      <View style={styles.centerContainer}>
        <ActivityIndicator size="large" color="#6200EE" />
      </View>
    );
  }

  if (!vault) {
    return (
      <View style={styles.centerContainer}>
        <Text style={styles.errorText}>Vault not found</Text>
        <TouchableOpacity style={styles.backBtn} onPress={onGoBack}>
          <Text style={styles.backBtnText}>Go Back</Text>
        </TouchableOpacity>
      </View>
    );
  }

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <TouchableOpacity onPress={onGoBack}>
        <Text style={styles.backLink}>Back</Text>
      </TouchableOpacity>

      <Text style={styles.title}>Vault Details</Text>

      {/* Status */}
      <View style={styles.card}>
        <Text style={styles.cardLabel}>Status</Text>
        <View
          style={[
            styles.statusBadge,
            { backgroundColor: statusColor[vault.status] ?? "#9E9E9E" },
          ]}
        >
          <Text style={styles.statusText}>
            {statusLabel[vault.status] ?? vault.status}
          </Text>
        </View>
      </View>

      {/* Balance */}
      <View style={styles.card}>
        <Text style={styles.cardLabel}>Balance</Text>
        {vault.balance && vault.balance.length > 0 ? (
          vault.balance.map((b: { token: string; amount: string }, i: number) => (
            <View key={i} style={styles.balanceRow}>
              <Text style={styles.balanceToken}>{b.token}</Text>
              <Text style={styles.balanceAmount}>{b.amount}</Text>
            </View>
          ))
        ) : (
          <Text style={styles.emptyText}>No balance yet</Text>
        )}
      </View>

      {/* Escrow */}
      {vault.escrowContract && (
        <View style={styles.card}>
          <Text style={styles.cardLabel}>Escrow Contract</Text>
          <Text style={styles.monoText} numberOfLines={1}>
            {vault.escrowContract}
          </Text>
        </View>
      )}

      {/* Beneficiaries */}
      <View style={styles.card}>
        <View style={styles.cardHeader}>
          <Text style={styles.cardLabel}>Beneficiaries</Text>
          <TouchableOpacity
            onPress={() => onNavigateManageBeneficiaries(vaultId)}
          >
            <Text style={styles.editLink}>Manage</Text>
          </TouchableOpacity>
        </View>

        {vault.beneficiaries && vault.beneficiaries.length > 0 ? (
          vault.beneficiaries.map(
            (b: { address: string; percentage: number; claimed: boolean }, i: number) => (
              <View key={i} style={styles.beneficiaryRow}>
                <Text style={styles.beneficiaryAddress} numberOfLines={1}>
                  {b.address}
                </Text>
                <View style={styles.beneficiaryMeta}>
                  <Text style={styles.beneficiaryPct}>
                    {(b.percentage / 100).toFixed(2)}%
                  </Text>
                  {b.claimed && (
                    <Text style={styles.claimedBadge}>Claimed</Text>
                  )}
                </View>
              </View>
            )
          )
        ) : (
          <Text style={styles.emptyText}>No beneficiaries set</Text>
        )}
      </View>

      {/* Demo Controls — Force Transition */}
      <View style={styles.card}>
        <Text style={styles.cardLabel}>Demo Controls</Text>
        <Text style={styles.demoHint}>
          Force vault status transition for demo purposes
        </Text>
        <View style={styles.transitionRow}>
          {["ACTIVE", "ALERT", "GRACE_PERIOD", "TRIGGERED"].map((status) => (
            <TouchableOpacity
              key={status}
              style={[
                styles.transitionBtn,
                { backgroundColor: statusColor[status] ?? "#9E9E9E" },
                vault.status === status && styles.transitionBtnDisabled,
              ]}
              onPress={() => handleForceTransition(status)}
              disabled={vault.status === status || transitioning}
            >
              <Text style={styles.transitionBtnText}>
                {statusLabel[status] ?? status}
              </Text>
            </TouchableOpacity>
          ))}
        </View>
        {transitioning && (
          <ActivityIndicator size="small" color="#6200EE" style={{ marginTop: 8 }} />
        )}
        {transitionError && (
          <Text style={styles.transitionError}>{transitionError}</Text>
        )}
      </View>

      {/* Created At */}
      <View style={styles.card}>
        <Text style={styles.cardLabel}>Created</Text>
        <Text style={styles.dateText}>
          {new Date(vault.createdAt).toLocaleDateString()}
        </Text>
      </View>
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
  centerContainer: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: "#FAFAFA",
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
    marginBottom: 24,
  },
  card: {
    backgroundColor: "#FFFFFF",
    padding: 16,
    borderRadius: 12,
    elevation: 1,
    marginBottom: 12,
  },
  cardHeader: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: 8,
  },
  cardLabel: {
    fontSize: 12,
    color: "#757575",
    marginBottom: 8,
  },
  statusBadge: {
    alignSelf: "flex-start",
    paddingHorizontal: 16,
    paddingVertical: 6,
    borderRadius: 12,
  },
  statusText: {
    color: "#FFFFFF",
    fontSize: 14,
    fontWeight: "600",
  },
  balanceRow: {
    flexDirection: "row",
    justifyContent: "space-between",
    paddingVertical: 4,
  },
  balanceToken: {
    fontSize: 14,
    color: "#757575",
  },
  balanceAmount: {
    fontSize: 14,
    fontWeight: "600",
    color: "#212121",
  },
  monoText: {
    fontSize: 12,
    fontFamily: "monospace",
    color: "#212121",
  },
  editLink: {
    fontSize: 14,
    color: "#6200EE",
    fontWeight: "600",
  },
  beneficiaryRow: {
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: "#F5F5F5",
  },
  beneficiaryAddress: {
    fontSize: 12,
    fontFamily: "monospace",
    color: "#212121",
    marginBottom: 4,
  },
  beneficiaryMeta: {
    flexDirection: "row",
    alignItems: "center",
  },
  beneficiaryPct: {
    fontSize: 14,
    fontWeight: "600",
    color: "#6200EE",
  },
  claimedBadge: {
    fontSize: 12,
    color: "#4CAF50",
    fontWeight: "600",
    marginLeft: 12,
  },
  emptyText: {
    fontSize: 14,
    color: "#BDBDBD",
    fontStyle: "italic",
  },
  dateText: {
    fontSize: 14,
    color: "#212121",
  },
  demoHint: {
    fontSize: 12,
    color: "#BDBDBD",
    fontStyle: "italic",
    marginBottom: 12,
  },
  transitionRow: {
    flexDirection: "row",
    flexWrap: "wrap",
    gap: 8,
  },
  transitionBtn: {
    paddingHorizontal: 14,
    paddingVertical: 8,
    borderRadius: 16,
  },
  transitionBtnDisabled: {
    opacity: 0.3,
  },
  transitionBtnText: {
    color: "#FFFFFF",
    fontSize: 12,
    fontWeight: "600",
  },
  transitionError: {
    fontSize: 12,
    color: "#F44336",
    marginTop: 8,
  },
  errorText: {
    fontSize: 16,
    color: "#F44336",
    marginBottom: 16,
  },
  backBtn: {
    backgroundColor: "#757575",
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 20,
  },
  backBtnText: {
    color: "#FFFFFF",
    fontSize: 14,
    fontWeight: "600",
  },
});
