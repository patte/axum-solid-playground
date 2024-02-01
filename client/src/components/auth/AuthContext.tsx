import { createContext, createSignal, useContext } from "solid-js";

type Session = {
  username: string;
};

export const AuthContext = createContext<{
  session: () => Session | null;
  signIn: (username: string) => void;
  signOut: () => void;
  isSignedIn: () => boolean;
}>();

const [session, setSession] = createSignal<Session | null>(null);

export const authContext = {
  session: () => session(),
  signIn: (username: string) => {
    setSession({ username });
  },
  signOut: () => {
    setSession(null);
  },
  isSignedIn: () => !!session(),
};

export const useAuth = () => useContext(AuthContext)!;

export const AuthProvider = (props: any) => (
  <AuthContext.Provider value={authContext}>
    {props.children}
  </AuthContext.Provider>
);
