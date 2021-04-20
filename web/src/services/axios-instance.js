import axios from "axios";
import store from "../store";

const $http = axios.create({
  baseURL: process.env.VUE_APP_AXIOS_BASE || "http://localhost:3001",
});

$http.interceptors.request.use(function(config) {
  const token = $cookies.get("token");
  config.headers.Authorization = `Bearer ${token}`;
  return config;
});

$http.interceptors.response.use(undefined, function(error) {
  if (error) {
    const originalRequest = error.config;
    if (
      (error.response.status === 401 && !originalRequest._retry) ||
      (originalRequest.url === "api/users" && error.response.status === 404)
    ) {
      originalRequest._retry = true;
      store.dispatch("logout");
      return router.push("/login");
    }
  }
});

export default $http;
