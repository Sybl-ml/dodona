import axios from 'axios';

const $http = axios.create({
    baseURL: process.env.VUE_APP_AXIOS_BASE || "http://localhost:3001"
});

$http.interceptors.request.use(function (config) {
    const token = $cookies.get("token");
    config.headers.Authorization = `Bearer ${token}`;
    return config;
});

export default $http;