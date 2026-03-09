/// Apollo Client setup for Pulse Protocol MVP.
/// HTTP-only (no WebSocket subscriptions in MVP).

import { ApolloClient, InMemoryCache, createHttpLink } from "@apollo/client";
import { setContext } from "@apollo/client/link/context";
import { useAuthStore } from "../../stores/authStore";

const API_URL = __DEV__
  ? "http://localhost:8080/graphql"
  : "https://api.pulseprotocol.io/graphql";

const httpLink = createHttpLink({
  uri: API_URL,
});

const authLink = setContext((_, { headers }) => {
  const token = useAuthStore.getState().sessionToken;
  return {
    headers: {
      ...headers,
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
  };
});

export const apolloClient = new ApolloClient({
  link: authLink.concat(httpLink),
  cache: new InMemoryCache(),
  defaultOptions: {
    watchQuery: { fetchPolicy: "cache-and-network" },
    query: { fetchPolicy: "network-only" },
  },
});
