/// WelcomeScreen — Entry point of the app.
/// Shows branding and a button to create a new Stellar wallet.

import React from "react";
import { View, Text, TouchableOpacity, StyleSheet } from "react-native";

type Props = {
  onCreateWallet: () => void;
};

export default function WelcomeScreen({ onCreateWallet }: Props) {
  return (
    <View style={styles.container}>
      <View style={styles.brandContainer}>
        <Text style={styles.logo}>PULSE</Text>
        <Text style={styles.logoSub}>PROTOCOL</Text>
        <Text style={styles.tagline}>
          Decentralized Cryptographic Inheritance
        </Text>
      </View>

      <View style={styles.infoContainer}>
        <Text style={styles.infoText}>
          Secure your digital assets for your loved ones with AI-powered
          proof-of-life verification on Stellar.
        </Text>
      </View>

      <TouchableOpacity style={styles.createButton} onPress={onCreateWallet}>
        <Text style={styles.createButtonText}>Create Wallet</Text>
      </TouchableOpacity>

      <Text style={styles.footer}>Built on Stellar / Soroban</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 32,
    backgroundColor: "#1A1A2E",
    alignItems: "center",
    justifyContent: "center",
  },
  brandContainer: {
    alignItems: "center",
    marginBottom: 48,
  },
  logo: {
    fontSize: 48,
    fontWeight: "bold",
    color: "#6200EE",
    letterSpacing: 8,
  },
  logoSub: {
    fontSize: 18,
    color: "#BB86FC",
    letterSpacing: 12,
    marginTop: -4,
  },
  tagline: {
    fontSize: 14,
    color: "#AAAAAA",
    marginTop: 16,
    textAlign: "center",
  },
  infoContainer: {
    marginBottom: 48,
    paddingHorizontal: 16,
  },
  infoText: {
    fontSize: 16,
    color: "#CCCCCC",
    textAlign: "center",
    lineHeight: 24,
  },
  createButton: {
    backgroundColor: "#6200EE",
    paddingHorizontal: 56,
    paddingVertical: 18,
    borderRadius: 28,
    elevation: 4,
  },
  createButtonText: {
    color: "#FFFFFF",
    fontSize: 18,
    fontWeight: "600",
  },
  footer: {
    position: "absolute",
    bottom: 32,
    fontSize: 12,
    color: "#666666",
  },
});
