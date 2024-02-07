import { createContext, createSignal, useContext } from "solid-js";
import { User } from "~/lib/auth";

export const AuthContext = createContext<{
  me: () => User | null;
  signIn: (user: User) => void;
  signOut: () => void;
  isSignedIn: () => boolean;
}>();

const [me, setMe] = createSignal<User | null>(null);

export const useAuth = () => useContext(AuthContext)!;

// auth.rs sets the informative cookie for the client
//
function get_me_from_cookie() {
  const cookie_user = document?.cookie
    ? document.cookie
        ?.split(";")
        .map((v) => v.trim())
        .find((v) => v.startsWith("authenticated_user_js"))
        ?.split("=")[1]
    : null;
  let me_from_cookie;
  if (cookie_user) {
    try {
      me_from_cookie = JSON.parse(cookie_user);
    } catch (e) {
      me_from_cookie = null;
    }
  }
  return me_from_cookie;
}

export const AuthProvider = (props: any) => {
  const me_from_cookie = get_me_from_cookie();
  if (!me() && me_from_cookie) {
    setMe(me_from_cookie);
  }
  const authContext = {
    me: () => me(),
    signIn: (user: User) => {
      setMe(user);
    },
    signOut: async () => {
      fetch("/signout", {
        method: "POST",
      });
      setMe(null);
    },
    isSignedIn: () => !!me(),
  };
  return (
    <AuthContext.Provider value={authContext}>
      {props.children}
    </AuthContext.Provider>
  );
};
