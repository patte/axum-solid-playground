import {
  GraphQLClientQuery,
  createGraphQLClient,
} from "@solid-primitives/graphql";
import {
  Accessor,
  createContext,
  createMemo,
  createSignal,
  useContext,
} from "solid-js";

export const GraphQLContext = createContext<Accessor<GraphQLClientQuery>>();

function createClient() {
  return createGraphQLClient(`${window.location.origin}/graphql`, {
    credentials: "same-origin",
  });
}

export function useGraphQL() {
  const context = useContext(GraphQLContext);
  if (!context) {
    throw new Error("useGraphQL: cannot find a GraphQLContext");
  }
  return context;
}

export function GraphQLProvider(props: { children: any }) {
  const client = createMemo(() => createClient());
  return (
    <GraphQLContext.Provider value={client}>
      {props.children}
    </GraphQLContext.Provider>
  );
}
