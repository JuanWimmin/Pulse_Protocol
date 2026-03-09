/// DashboardScreen — Main screen showing liveness score + vault summary.
/// Uses MY_VAULTS_QUERY + MY_LIVENESS_QUERY from Apollo Client.

import React, { useEffect } from "react";
import {
  View,
  Text,
  TouchableOpacity,
  ScrollView,
  ActivityIndicator,
  StyleSheet,
  RefreshControl,
} from "react-native";
import { useQuery } from "@apollo/client";
import { MY_VAULTS_QUERY, MY_LIVENESS_QUERY } from "../services/graphql/queries";
import { useVaultStore } from "../stores/vaultStore";
import { useAuthStore } from "../stores/authStore";
import { VaultStatus } from "../types/verification";

type Props = {
  onNavigateVerify: () => void;
  onNavigateCreateVault: () => void;
  onNavigateVaultDetail: (vaultId: string) => void;
  onNavigateEmergency: () => void;
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

export default function DashboardScreen({
  onNavigateVerify,
  onNavigateCreateVault,
  onNavigateVaultDetail,
  onNavigateEmergency,
}: Props) {
  const stellarAddress = useAuthStore((s) => s.stellarAddress);
  const { currentVault, livenessScore, setVaults, setLiveness } = useVaultStore();

  const {
    data: vaultsData,
    loading: vaultsLoading,
    refetch: refetchVaults,
  } = useQuery(MY_VAULTS_QUERY, { fetchPolicy: "cache-and-network" });

  const {
    data: livenessData,
    loading: livenessLoading,
    refetch: refetchLiveness,
  } = useQuery(MY_LIVENESS_QUERY, { fetchPolicy: "cache-and-network" });

  useEffect(() => {
    if (vaultsData?.myVaults) {
      setVaults(vaultsData.myVaults);
    }
  }, [vaultsData, setVaults]);

  useEffect(() => {
    if (livenessData?.myLiveness) {
      setLiveness(
        livenessData.myLiveness.score,
        livenessData.myLiveness.lastVerified
      );
    }
  }, [livenessData, setLiveness]);

  const score = livenessScore ?? 0;
  const scorePercent = (score / 100).toFixed(2);
  const scoreColorValue =
    score >= 7000 ? "#4CAF50" : score >= 3000 ? "#FF9800" : "#F44336";

  const isLoading = vaultsLoading || livenessLoading;
  const vaultStatus = currentVault?.status ?? VaultStatus.ACTIVE;
  const showEmergency =
    vaultStatus === VaultStatus.ALERT ||
    vaultStatus === VaultStatus.GRACE_PERIOD;

  const onRefresh = () => {
    refetchVaults();
    refetchLiveness();
  };

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.content}
      refreshControl={
        <RefreshControl refreshing={isLoading} onRefresh={onRefresh} />
      }
    >
      <Text style={styles.greeting}>Pulse Protocol</Text>
      <Text style={styles.address} numberOfLines={1}>
        {stellarAddress ?? "No wallet"}
      </Text>

      {/* Liveness Score Card */}
      <View style={styles.scoreCard}>
        <Text style={styles.scoreLabel}>Liveness Score</Text>
        {livenessLoading ? (
          <ActivityIndicator size="small" color="#6200EE" />
        ) : (
          <>
            <Text style={[styles.scoreValue, { color: scoreColorValue }]}>
              {scorePercent}%
            </Text>
            <Text style={styles.scoreRaw}>{score} / 10000</Text>
          </>
        )}
        <TouchableOpacity style={styles.verifyButton} onPress={onNavigateVerify}>
          <Text style={styles.verifyButtonText}>Verify Life</Text>
        </TouchableOpacity>
      </View>

      {/* Vault Status Card */}
      <View style={styles.vaultCard}>
        <Text style={styles.cardTitle}>My Vault</Text>
        {vaultsLoading ? (
          <ActivityIndicator size="small" color="#6200EE" />
        ) : currentVault ? (
          <TouchableOpacity
            onPress={() => onNavigateVaultDetail(currentVault.id)}
          >
            <View style={styles.vaultRow}>
              <View
                style={[
                  styles.statusBadge,
                  { backgroundColor: statusColor[vaultStatus] ?? "#9E9E9E" },
                ]}
              >
                <Text style={styles.statusBadgeText}>
                  {statusLabel[vaultStatus] ?? vaultStatus}
                </Text>
              </View>
              <Text style={styles.beneficiaryCount}>
                {currentVault.beneficiaries.length} beneficiaries
              </Text>
            </View>
            <Text style={styles.tapHint}>Tap to view details</Text>
          </TouchableOpacity>
        ) : (
          <View style={styles.noVaultContainer}>
            <Text style={styles.noVaultText}>No vault created yet</Text>
            <TouchableOpacity
              style={styles.createButton}
              onPress={onNavigateCreateVault}
            >
              <Text style={styles.createButtonText}>Create Vault</Text>
            </TouchableOpacity>
          </View>
        )}
      </View>

      {/* Emergency Check-in */}
      {showEmergency && (
        <TouchableOpacity
          style={styles.emergencyCard}
          onPress={onNavigateEmergency}
        >
          <Text style={styles.emergencyTitle}>Emergency Check-In Available</Text>
          <Text style={styles.emergencyDesc}>
            Your vault is in {statusLabel[vaultStatus]}. Tap to verify and reset.
          </Text>
        </TouchableOpacity>
      )}
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
  greeting: {
    fontSize: 24,
    fontWeight: "bold",
    color: "#212121",
    marginBottom: 4,
  },
  address: {
    fontSize: 12,
    fontFamily: "monospace",
    color: "#757575",
    marginBottom: 24,
  },
  scoreCard: {
    backgroundColor: "#FFFFFF",
    padding: 24,
    borderRadius: 16,
    alignItems: "center",
    elevation: 2,
    marginBottom: 16,
  },
  scoreLabel: {
    fontSize: 14,
    color: "#757575",
    marginBottom: 8,
  },
  scoreValue: {
    fontSize: 48,
    fontWeight: "bold",
  },
  scoreRaw: {
    fontSize: 12,
    color: "#BDBDBD",
    marginBottom: 20,
  },
  verifyButton: {
    backgroundColor: "#6200EE",
    paddingHorizontal: 36,
    paddingVertical: 12,
    borderRadius: 24,
  },
  verifyButtonText: {
    color: "#FFFFFF",
    fontSize: 16,
    fontWeight: "600",
  },
  vaultCard: {
    backgroundColor: "#FFFFFF",
    padding: 20,
    borderRadius: 16,
    elevation: 2,
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: "600",
    color: "#212121",
    marginBottom: 12,
  },
  vaultRow: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
  },
  statusBadge: {
    paddingHorizontal: 16,
    paddingVertical: 6,
    borderRadius: 12,
  },
  statusBadgeText: {
    color: "#FFFFFF",
    fontSize: 13,
    fontWeight: "600",
  },
  beneficiaryCount: {
    fontSize: 14,
    color: "#757575",
  },
  tapHint: {
    fontSize: 12,
    color: "#BDBDBD",
    marginTop: 8,
    textAlign: "center",
  },
  noVaultContainer: {
    alignItems: "center",
  },
  noVaultText: {
    fontSize: 14,
    color: "#757575",
    marginBottom: 16,
  },
  createButton: {
    backgroundColor: "#6200EE",
    paddingHorizontal: 32,
    paddingVertical: 12,
    borderRadius: 24,
  },
  createButtonText: {
    color: "#FFFFFF",
    fontSize: 14,
    fontWeight: "600",
  },
  emergencyCard: {
    backgroundColor: "#FFF3E0",
    padding: 20,
    borderRadius: 16,
    borderWidth: 1,
    borderColor: "#FF9800",
    marginBottom: 16,
  },
  emergencyTitle: {
    fontSize: 16,
    fontWeight: "600",
    color: "#E65100",
    marginBottom: 4,
  },
  emergencyDesc: {
    fontSize: 14,
    color: "#BF360C",
  },
});
