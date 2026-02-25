/// Biometric fingerprint module wrapping react-native-biometrics.
///
/// Provides a clean interface for:
///   - Checking biometric availability on the device
///   - Authenticating the user via BiometricPrompt (fingerprint/face)
///
/// Never stores raw biometric data — only captures success/failure + timestamp.

import ReactNativeBiometrics, {
  BiometryType,
} from "react-native-biometrics";

export interface BiometricAvailability {
  available: boolean;
  biometryType: BiometryType | undefined;
}

export interface BiometricAuthResult {
  success: boolean;
  timestamp: number;
  error?: string;
}

const rnBiometrics = new ReactNativeBiometrics();

/// Check if biometric authentication is available on this device.
export async function checkBiometricAvailability(): Promise<BiometricAvailability> {
  try {
    const { available, biometryType } =
      await rnBiometrics.isSensorAvailable();
    return { available, biometryType };
  } catch {
    return { available: false, biometryType: undefined };
  }
}

/// Prompt the user for biometric authentication (fingerprint or face).
/// Returns success status and the timestamp of the authentication attempt.
export async function authenticateWithBiometric(
  promptMessage = "Verify your identity"
): Promise<BiometricAuthResult> {
  try {
    const { success } = await rnBiometrics.simplePrompt({
      promptMessage,
      cancelButtonText: "Cancel",
    });

    return {
      success,
      timestamp: Date.now(),
    };
  } catch (error: unknown) {
    const message =
      error instanceof Error ? error.message : "Biometric authentication failed";
    return {
      success: false,
      timestamp: Date.now(),
      error: message,
    };
  }
}
