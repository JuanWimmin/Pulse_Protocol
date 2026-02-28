import { signMessage } from "@stellar/freighter-api";
import { Buffer } from "buffer";

export const API_URL =
  import.meta.env.VITE_API_URL?.toString() || "http://localhost:8080";

export type AuthResult = {
  token: string;
  stellar_address: string;
};

export type GraphQLError = {
  message: string;
};

export type GraphQLResponse<T> = {
  data?: T;
  errors?: GraphQLError[];
};

export async function authenticateWithWallet(
  address: string
): Promise<AuthResult> {
  const timestamp = Date.now();
  const message = `pulse-auth:${timestamp}`;
  const signed = await signMessage(message, { address });

  if ("error" in signed && signed.error) {
    throw new Error(signed.error.message || "Failed to sign message");
  }

  const signatureHex = normalizeSignatureToHex(signed.signedMessage);

  const response = await fetch(`${API_URL}/auth`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      stellar_address: address,
      message,
      signature: signatureHex
    })
  });

  if (!response.ok) {
    const payload = await response.json().catch(() => ({}));
    throw new Error(payload?.error || "Auth failed");
  }

  return response.json() as Promise<AuthResult>;
}

export async function graphQLRequest<T>(
  token: string,
  query: string,
  variables?: Record<string, unknown>
): Promise<T> {
  const response = await fetch(`${API_URL}/graphql`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`
    },
    body: JSON.stringify({ query, variables })
  });

  const payload = (await response.json()) as GraphQLResponse<T>;

  if (!response.ok || payload.errors?.length) {
    const errorMessage =
      payload.errors?.[0]?.message || "GraphQL request failed";
    throw new Error(errorMessage);
  }

  if (!payload.data) {
    throw new Error("Empty response from API");
  }

  return payload.data;
}

function normalizeSignatureToHex(signature: string | Buffer | null): string {
  if (!signature) {
    throw new Error("Signature missing");
  }

  if (typeof signature !== "string") {
    return Buffer.from(signature).toString("hex");
  }

  const cleaned = signature.trim();
  const isHex = /^[0-9a-fA-F]+$/.test(cleaned);
  if (isHex && cleaned.length === 128) {
    return cleaned.toLowerCase();
  }

  try {
    return Buffer.from(cleaned, "base64").toString("hex");
  } catch {
    throw new Error("Unsupported signature format");
  }
}
