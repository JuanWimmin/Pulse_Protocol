/// AppNavigator — Root navigation for Pulse Protocol MVP.
///
/// 3 navigators + 1 onboarding:
///   AuthNavigator: Welcome → CreateWallet
///   SimpleOnboardingNavigator: BiometricSetup
///   MainNavigator: Dashboard, CreateVault, VaultDetail, ManageBeneficiaries, Verify, EmergencyCheckin
///   ClaimNavigator: Claim

import React from "react";
import { createStackNavigator } from "@react-navigation/stack";
import { useAuthStore } from "../stores/authStore";

import WelcomeScreen from "../screens/WelcomeScreen";
import CreateWalletScreen from "../screens/CreateWalletScreen";
import BiometricSetupScreen from "../screens/BiometricSetupScreen";
import DashboardScreen from "../screens/DashboardScreen";
import CreateVaultScreen from "../screens/CreateVaultScreen";
import VaultDetailScreen from "../screens/VaultDetailScreen";
import ManageBeneficiariesScreen from "../screens/ManageBeneficiariesScreen";
import VerifyScreen from "../screens/VerifyScreen";
import EmergencyCheckinScreen from "../screens/EmergencyCheckinScreen";
import ClaimScreen from "../screens/ClaimScreen";
import { useVaultStore } from "../stores/vaultStore";
import { VaultStatus } from "../types/verification";

export type RootStackParamList = {
  Welcome: undefined;
  CreateWallet: undefined;
  BiometricSetup: undefined;
  Dashboard: undefined;
  CreateVault: undefined;
  VaultDetail: { vaultId: string };
  ManageBeneficiaries: { vaultId: string };
  Verify: undefined;
  EmergencyCheckin: undefined;
  Claim: undefined;
};

const Stack = createStackNavigator<RootStackParamList>();

export default function AppNavigator() {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const biometricSetupComplete = useAuthStore((s) => s.biometricSetupComplete);

  return (
    <Stack.Navigator screenOptions={{ headerShown: false }}>
      {!isAuthenticated ? (
        // Auth flow
        <>
          <Stack.Screen name="Welcome">
            {({ navigation }) => (
              <WelcomeScreen
                onCreateWallet={() => navigation.navigate("CreateWallet")}
              />
            )}
          </Stack.Screen>
          <Stack.Screen name="CreateWallet">
            {({ navigation }) => (
              <CreateWalletScreen
                onWalletCreated={() => navigation.navigate("BiometricSetup")}
              />
            )}
          </Stack.Screen>
        </>
      ) : !biometricSetupComplete ? (
        // Onboarding flow
        <Stack.Screen name="BiometricSetup">
          {() => (
            <BiometricSetupScreen
              onSetupComplete={() => useAuthStore.getState().setBiometricSetup(true)}
            />
          )}
        </Stack.Screen>
      ) : (
        // Main app
        <>
          <Stack.Screen name="Dashboard">
            {({ navigation }) => (
              <DashboardScreen
                onNavigateVerify={() => navigation.navigate("Verify")}
                onNavigateCreateVault={() => navigation.navigate("CreateVault")}
                onNavigateVaultDetail={(vaultId) =>
                  navigation.navigate("VaultDetail", { vaultId })
                }
                onNavigateEmergency={() =>
                  navigation.navigate("EmergencyCheckin")
                }
              />
            )}
          </Stack.Screen>
          <Stack.Screen name="CreateVault">
            {({ navigation }) => (
              <CreateVaultScreen
                onVaultCreated={(vaultId) =>
                  navigation.navigate("VaultDetail", { vaultId })
                }
                onGoBack={() => navigation.goBack()}
              />
            )}
          </Stack.Screen>
          <Stack.Screen name="VaultDetail">
            {({ navigation, route }) => (
              <VaultDetailScreen
                vaultId={route.params.vaultId}
                onNavigateManageBeneficiaries={(vaultId) =>
                  navigation.navigate("ManageBeneficiaries", { vaultId })
                }
                onGoBack={() => navigation.goBack()}
              />
            )}
          </Stack.Screen>
          <Stack.Screen name="ManageBeneficiaries">
            {({ navigation, route }) => (
              <ManageBeneficiariesScreen
                vaultId={route.params.vaultId}
                onGoBack={() => navigation.goBack()}
              />
            )}
          </Stack.Screen>
          <Stack.Screen name="Verify">
            {({ navigation }) => {
              const lastVerified = useVaultStore.getState().lastVerified;
              return (
                <VerifyScreen
                  lastVerificationTimestamp={
                    lastVerified ? new Date(lastVerified).getTime() : null
                  }
                  onVerificationComplete={() => navigation.goBack()}
                />
              );
            }}
          </Stack.Screen>
          <Stack.Screen name="EmergencyCheckin">
            {({ navigation }) => {
              const vault = useVaultStore.getState().currentVault;
              return (
                <EmergencyCheckinScreen
                  currentVaultStatus={vault?.status ?? VaultStatus.ACTIVE}
                  lastVerificationTimestamp={null}
                  onCheckinComplete={() => navigation.goBack()}
                  onGoBack={() => navigation.goBack()}
                />
              );
            }}
          </Stack.Screen>
          <Stack.Screen name="Claim">
            {({ navigation }) => (
              <ClaimScreen onGoBack={() => navigation.goBack()} />
            )}
          </Stack.Screen>
        </>
      )}
    </Stack.Navigator>
  );
}
