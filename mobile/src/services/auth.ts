/// Auth service for Pulse Protocol MVP.
/// Handles Stellar signature-based authentication with the backend.

import { Keypair } from "@stellar/stellar-sdk";
import { Buffer } from "buffer";

export const API_BASE_URL = __DEV__
  ? "http://localhost:8080"
  : "https://api.pulseprotocol.io";

interface AuthResult {
  token: string;
  stellar_address: string;
}

export async function authenticateWithBackend(
  keypair: Keypair
): Promise<AuthResult> {
  const message = `pulse-auth:${Date.now()}`;
  const signature = keypair.sign(Buffer.from(message));
  const signatureHex = Buffer.from(signature).toString("hex");

  const response = await fetch(`${API_BASE_URL}/auth`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      stellar_address: keypair.publicKey(),
      message,
      signature: signatureHex,
    }),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: "Auth failed" }));
    throw new Error(error.error ?? `Auth failed: ${response.status}`);
  }

  return response.json();
}
