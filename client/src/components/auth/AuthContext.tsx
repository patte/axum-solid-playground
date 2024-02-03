import { createContext, createSignal, useContext } from "solid-js";
import { User } from "~/lib/auth";

export const AuthContext = createContext<{
  me: () => User | null;
  signIn: (user: User) => void;
  signOut: () => void;
  isSignedIn: () => boolean;
}>();

const [me, setMe] = createSignal<User | null>(null);

export const authContext = {
  me: () => me(),
  signIn: (user: User) => {
    setMe(user);
  },
  signOut: () => {
    setMe(null);
  },
  isSignedIn: () => !!me(),
};

export const useAuth = () => useContext(AuthContext)!;

export const AuthProvider = (props: any) => (
  <AuthContext.Provider value={authContext}>
    {props.children}
  </AuthContext.Provider>
);
