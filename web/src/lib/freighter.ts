import {
  getAddress,
  getNetwork,
  isConnected,
  requestAccess
} from "@stellar/freighter-api";

export type FreighterStatus = {
  installed: boolean;
  address?: string;
  network?: string;
  error?: string;
};

export async function checkFreighter(): Promise<FreighterStatus> {
  try {
    const connection = await isConnected();
    if (!connection.isConnected) {
      return { installed: false };
    }

    const addressObj = await getAddress().catch(() => undefined);
    const networkObj = await getNetwork().catch(() => undefined);
    const address =
      addressObj && "address" in addressObj
        ? addressObj.address
        : addressObj && "publicKey" in addressObj
          ? addressObj.publicKey
          : undefined;
    const network = networkObj && "network" in networkObj ? networkObj.network : undefined;

    return {
      installed: true,
      address,
      network
    };
  } catch (error) {
    return {
      installed: false,
      error: error instanceof Error ? error.message : "Unknown error"
    };
  }
}

export async function connectFreighter(): Promise<FreighterStatus> {
  try {
    const connection = await isConnected();
    if (!connection.isConnected) {
      return { installed: false };
    }

    const access = await requestAccess();
    if (access.error) {
      return {
        installed: true,
        error: access.error
      };
    }

    const address =
      "address" in access
        ? access.address
        : "publicKey" in access
          ? access.publicKey
          : undefined;
    const networkObj = await getNetwork().catch(() => undefined);
    const network = networkObj && "network" in networkObj ? networkObj.network : undefined;

    return {
      installed: true,
      address,
      network
    };
  } catch (error) {
    return {
      installed: false,
      error: error instanceof Error ? error.message : "Unknown error"
    };
  }
}
