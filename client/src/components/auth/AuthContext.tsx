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

function get_me_from_cookie(): {
  me_from_cookie: User | null;
  expiry_date: Date | null;
} {
  const cookiePayload = document?.cookie
    ? document.cookie
        ?.split(";")
        .map((v) => v.trim())
        .find((v) => v.startsWith("authenticated_user_js"))
        ?.split("=")[1]
    : null;
  let me_from_cookie = null;
  let expiry_date = null;
  if (cookiePayload) {
    try {
      let parsed_cookie = JSON.parse(cookiePayload);
      me_from_cookie = parsed_cookie.user;
      expiry_date = new Date(parsed_cookie.expiry_date);
    } catch (e) {}
  }
  return {
    me_from_cookie,
    expiry_date,
  };
}

let refreshTimeout: any;

// keeps me() updated with the cookie
// sets a timeout to refresh itself when the cookie expires
// if another request refreshed the cookie the timeout is cleared and reset
// api requests should refresh the cookie
// if no activity happens, the client will reactively "sign out" on expiry
function setMeFromCookie() {
  const { me_from_cookie, expiry_date } = get_me_from_cookie();
  if (me_from_cookie) {
    setMe(me_from_cookie);
    clearTimeout(refreshTimeout);
    refreshTimeout = setTimeout(
      setMeFromCookie,
      expiry_date!.getTime() - new Date().getTime()
    );
  } else if (me()) {
    setMe(null);
  }
}

export const AuthProvider = (props: any) => {
  setMeFromCookie();
  const authContext = {
    me: () => me(),
    signIn: (_user: User) => {
      setMeFromCookie();
    },
    signOut: async () => {
      fetch("/signout", {
        method: "POST",
      }).then(() => {
        setMeFromCookie();
      });
    },
    isSignedIn: () => !!me(),
  };
  return (
    <AuthContext.Provider value={authContext}>
      {props.children}
    </AuthContext.Provider>
  );
};
