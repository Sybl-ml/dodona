import Vue from "vue";
import Vuex from "vuex";
import createPersistedState from "vuex-persistedstate";
import $http from "../services/axios-instance";
import _ from "lodash";
import Papa from "papaparse";
import router from "../router";

import projects from "./modules/projects";
import models from "./modules/models";
Vue.use(Vuex);

export default new Vuex.Store({
  // plugins: [createPersistedState()],
  modules: {
    projects,
    models,
    // user_data,
    // socket,
  },
  state: {
    // projects: [],
    // models: [],
    user_data: {},
    socket: {
      isConnected: false,
      authenticated: false,
      message: "",
      reconnectError: false,
    },
  },
  getters: {
    isAuthenticated: (state) => {
      return !_.isEmpty(state.user_data);
    },
  },
  mutations: {
    SOCKET_ONOPEN(state, event) {
      Vue.prototype.$socket = event.currentTarget;
      state.socket.isConnected = true;

      let token = Vue.prototype.$cookies.get("token");
      let auth = {
        authentication: { token: token },
      };
      console.log("sending auth msg");
      Vue.prototype.$socket.sendObj(auth);
    },
    SOCKET_ONCLOSE(state, event) {
      state.socket.isConnected = false;
      state.socket.authenticated = false;
    },
    SOCKET_ONMESSAGE(state, message) {
      state.socket.message = message;

      console.log(message);
      switch (Object.keys(message)[0]) {
        case "hello":
          break;
        case "modelComplete":
          let {
            project_id,
            cluster_size,
            model_complete_count,
            success,
          } = message.modelComplete;

          projects.commit("updateProjectProgress", {
            project_id: project_id,
            success: success,
          });

          break;
        default:
          console.err("Unknown Message");
      }
    },
    setUser(state, user) {
      Vue.set(state, "user_data", user);
    },
    updateClientStatus(state) {
      Vue.set(state.user_data, "client", true);
    },
    setAvatar(state, avatar) {
      Vue.set(state.user_data, "avatar", avatar);
    },
  },
  actions: {
    sendMsg(context, msg) {
      Vue.prototype.$socket.sendObj(msg);
    },
    async getUserData(context) {
      if (context.user_data) {
        return;
      }
      return $http.get(`api/users`);
    },
    async getAvatar({ commit }) {
      let response = await $http.get(`api/users/avatar`);
      commit("setAvatar", response.data.img);
    },
    async postNewAvatar(context, avatar) {
      try {
        await $http.post("api/users/avatar", {
          avatar: avatar,
        });
      } catch (err) {
        console.log(err);
      }

      context.commit("setAvatar", avatar);
    },
    async login({ commit }, { email, password }) {
      return $http.post("api/users/login", {
        email: email,
        password: password,
      });
    },
    async logout({ commit }) {
      Vue.prototype.$cookies.remove("token");
      commit("setUser", {});
      router.push("/login");
    },
    async register(
      { commit },
      { email, password, firstName, lastName, currency, dob }
    ) {
      return $http.post("api/users/new", {
        email: email,
        password: password,
        firstName: firstName,
        lastName: lastName,
        currency: currency,
        dob: dob,
      });
    },
    async generatePrivateKey({ commit }) {
      console.log("Generating new private key");
      return $http.post("api/clients/generatePrivateKey");
    },
    async client_register({ commit }, { id, email, password }) {
      let response = await $http.post("api/clients/register", {
        id: id,
        email: email,
        password: password,
      });
      commit("updateClientStatus");
      return response;
    },
    async uploadAvatar(context, avatar) {
      return $http.post("api/users/avatar", {
        avatar,
      });
    },
  },
});
