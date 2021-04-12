import axios from "axios";

const $http = axios.create({
  baseURL: process.env.VUE_APP_AXIOS_BASE || "http://localhost:3001",
});

$http.interceptors.request.use(function(config) {
  const token = $cookies.get("token");
  config.headers.Authorization = `Bearer ${token}`;
  return config;
});

axios.interceptors.response.use(undefined, function(error) {
  if (error) {
    const originalRequest = error.config;
    if (error.response.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;
      store.dispatch("logout");
      return router.push("/login");
    }
  }
});

export default $http;
