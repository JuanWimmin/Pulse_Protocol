/// App.tsx — Root component for Pulse Protocol Mobile.
/// Sets up Apollo Client provider and Navigation container.

import React from "react";
import { NavigationContainer } from "@react-navigation/native";
import { ApolloProvider } from "@apollo/client";
import { apolloClient } from "./services/graphql/client";
import AppNavigator from "./navigation/AppNavigator";

export default function App() {
  return (
    <ApolloProvider client={apolloClient}>
      <NavigationContainer>
        <AppNavigator />
      </NavigationContainer>
    </ApolloProvider>
  );
}
