import { type RouteConfig, index, route } from "@react-router/dev/routes";

export default [
  index("routes/home.tsx"),
  route("albums", "routes/albums.tsx"),
  route("sharing", "routes/sharing.tsx"),
  route("settings", "routes/settings.tsx"),
  route("profile", "routes/profile.tsx"),
  route("setup", "routes/setup.tsx"),
  route("login", "routes/login.tsx"),
] satisfies RouteConfig;
